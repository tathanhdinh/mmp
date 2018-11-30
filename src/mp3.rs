use std::io::{Read, Seek, SeekFrom};
use std::time::Duration;

pub(crate) struct Mp3Decoder<R: Read> {
    reader: simplemad::Decoder<R>,
    current_frame: simplemad::Frame,
    current_frame_channel: usize,
    current_frame_sample_pos: usize,
    pub current_time: u64,
}

fn is_mp3<R: Read + Seek>(data: &mut R) -> bool {
    let stream_pos = data.seek(SeekFrom::Current(0)).unwrap();
    let is_mp3 = simplemad::Decoder::decode(data.by_ref()).is_ok();
    data.seek(SeekFrom::Start(stream_pos)).unwrap();
    is_mp3
}

fn next_frame<R: Read>(decoder: &mut simplemad::Decoder<R>) -> simplemad::Frame {
    decoder
        .filter_map(|f| f.ok())
        .next()
        .unwrap_or_else(|| simplemad::Frame {
            bit_rate: 0,
            layer: Default::default(),
            mode: Default::default(),
            sample_rate: 44100,
            samples: vec![],
            position: Duration::from_secs(0),
            duration: Duration::from_secs(0),
        })
}

impl<R: Read + Seek> Mp3Decoder<R> {
    pub fn new(mut data: R) -> Result<Mp3Decoder<R>, R> {
        if !is_mp3(data.by_ref()) {
            return Err(data);
        }

        let mut reader = simplemad::Decoder::decode(data).unwrap();

        let current_frame = next_frame(&mut reader);
        let current_time = crate::to_millis(current_frame.duration);

        Ok(Mp3Decoder {
            reader,
            current_frame,
            current_frame_channel: 0,
            current_frame_sample_pos: 0,
            current_time,
        })
    }

    pub fn compute_duration(mut data: R) -> Option<Duration> {
        if !is_mp3(data.by_ref()) {
            return None;
        }

        let decoder = simplemad::Decoder::decode_headers(data).unwrap();

        Some(
            decoder
                .filter_map(|frame| match frame {
                    Ok(frame) => Some(frame.duration),
                    Err(_) => None,
                })
                .sum(),
        )
    }

    pub fn sample_rate(&self) -> u32 {
        self.current_frame.sample_rate
    }
}

fn next_sample<R: Read>(decoder: &mut Mp3Decoder<R>) -> Option<i16> {
    if decoder.current_frame.samples[0].len() == 0 {
        return None;
    }

    // getting the sample and converting it from fixed step to i16
    let sample = decoder.current_frame.samples[decoder.current_frame_channel]
        [decoder.current_frame_sample_pos];
    let sample = sample.to_i32() + (1 << (28 - 16));
    let sample = if sample >= 0x10000000 {
        0x10000000
    } else if sample <= -0x10000000 {
        -0x10000000
    } else {
        sample
    };
    let sample = sample >> (28 + 1 - 16);
    let sample = sample as i16;

    decoder.current_frame_channel += 1;

    if decoder.current_frame_channel < decoder.current_frame.samples[0].len() {
        return Some(sample);
    }

    decoder.current_frame_channel = 0;
    decoder.current_frame_sample_pos += 1;

    if decoder.current_frame_sample_pos < decoder.current_frame.samples[0].len() {
        return Some(sample);
    }

    decoder.current_frame = next_frame(&mut decoder.reader);
    decoder.current_frame_channel = 0;
    decoder.current_frame_sample_pos = 0;
    decoder.current_time += crate::to_millis(decoder.current_frame.duration);

    return Some(sample);
}

impl<R: Read> Iterator for Mp3Decoder<R> {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        next_sample(self)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.current_frame.samples[0].len(), None)
    }
}
