use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
    thread,
};

// use crossbeam::sync::SegQueue;
use crossbeam::queue::SegQueue;

use pulse_simple::Playback;

use crate::mp3::Mp3Decoder;

const BUFFER_SIZE: usize = 1000;
const DEFAULT_RATE: u32 = 44100;

enum Action {
    Load(PathBuf),
    Stop,
}

#[derive(Clone)]
struct EventLoop {
    queue: Arc<SegQueue<Action>>,
    playing: Arc<Mutex<bool>>,
}

impl EventLoop {
    fn new() -> Self {
        EventLoop {
            queue: Arc::new(SegQueue::new()),
            playing: Arc::new(Mutex::new(false)),
        }
    }
}

pub struct Player {
    app_state: Arc<Mutex<super::State>>,
    event_loop: EventLoop,
}

impl Player {
    pub(crate) fn new(app_state: Arc<Mutex<super::State>>) -> Self {
        fn iter_to_buffer<I: Iterator<Item = i16>>(
            iter: &mut I,
            buffer: &mut [[i16; 2]; BUFFER_SIZE],
        ) -> usize {
            let mut iter = iter.take(BUFFER_SIZE);
            let mut index = 0;
            while let Some(sample1) = iter.next() {
                if let Some(sample2) = iter.next() {
                    buffer[index][0] = sample1;
                    buffer[index][1] = sample2;
                }
                index += 1;
            }
            index
        }

        let event_loop = EventLoop::new();
        {
            let app_state = app_state.clone();
            let event_loop = event_loop.clone();
            thread::spawn(move || {
                let mut buffer = [[0; 2]; BUFFER_SIZE];
                let mut playback = Playback::new("MP3", "MP3 Playback", None, DEFAULT_RATE);
                let mut source = None;
                loop {
                    if let Some(action) = event_loop.queue.try_pop() {
                        match action {
                            self::Action::Load(path) => {
                                let file = File::open(path).unwrap();
                                source = Some(Mp3Decoder::new(BufReader::new(file)).unwrap());
                                let rate = source
                                    .as_ref()
                                    .map(|src| src.sample_rate())
                                    .unwrap_or(DEFAULT_RATE);
                                playback = Playback::new("MP3", "MP3 Playback", None, rate);

                                // app_state.lock().unwrap().stopped = false;
                                let mut guard = app_state.lock().unwrap();
                                guard.stopped = false;
                            }

                            self::Action::Stop => {}
                        }
                    } else if *event_loop.playing.lock().unwrap() {
                        let mut written = false;
                        if let Some(ref mut source) = source {
                            let size = iter_to_buffer(source, &mut buffer);
                            if size > 0 {
                                playback.write(&buffer[..size]);
                                written = true;
                            }
                        }

                        if !written {
                            app_state.lock().unwrap().stopped = true;
                            *event_loop.playing.lock().unwrap() = false;
                            source = None;
                        }
                    }
                }
            });
        }
        Player {
            app_state,
            event_loop,
        }
    }

    pub fn load<P: AsRef<Path>>(&self, path: P) {
        let path_buf = path.as_ref().to_path_buf();
        self.emit(Action::Load(path_buf));
        self.set_playing(true);
    }

    fn emit(&self, action: Action) {
        self.event_loop.queue.push(action);
    }

    fn set_playing(&self, playing: bool) {
        *self.event_loop.playing.lock().unwrap() = playing;
    }
}
