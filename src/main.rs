mod mp3;
mod player;
mod playlist;
mod toolbar;

use self::{playlist::Playlist, toolbar::MusicToolbar};

use gio::{ApplicationExt, ApplicationExtManual, ApplicationFlags};
use gtk::{
    prelude::Inhibit,
    Adjustment, AdjustmentExt, Application, ApplicationWindow, ApplicationWindowExt, ContainerExt,
    Continue, GtkWindowExt, Image, ImageExt, Label, LabelExt,
    Orientation::{Horizontal, Vertical},
    Scale, ScaleExt, SeparatorToolItem, ToolButton, ToolButtonExt, Toolbar, WidgetExt,
};
use std::{
    collections::HashMap,
    env,
    rc::Rc,
    sync::{Arc, Mutex},
    time::Duration,
};

// const PLAY_STOCK: &'static str = "gtk-media-play";

struct State {
    current_time: u64,
    durations: HashMap<String, u64>,
    stopped: bool,
}

struct App {
    adjustment: Adjustment,
    cover: Image,
    current_time_label: Label,
    duration_label: Label,
    playlist: Rc<Playlist>,
    state: Arc<Mutex<State>>,
    toolbar: MusicToolbar,
    window: ApplicationWindow,
}

impl App {
    fn new(app: &Application) -> Self {
        let app_window = ApplicationWindow::new(app);
        app_window.set_title("A minimal music player");

        let mt = MusicToolbar::new();
        let vbox = gtk::Box::new(Vertical, 0);
        vbox.add(&mt.toolbar);

        let state = Arc::new(Mutex::new(State {
            current_time: 0,
            durations: HashMap::new(),
            stopped: true,
        }));

        // add playlist
        let pl = Rc::new(Playlist::new(state.clone()));
        vbox.add(&pl.treeview);

        // add cover...
        let img = Image::new();
        vbox.add(&img);

        let hbox = gtk::Box::new(Horizontal, 10);
        vbox.add(&hbox);

        // add scale
        let adjustment = Adjustment::new(0.0, 0.0, 10.0, 0.0, 0.0, 0.0);

        let scale = Scale::new(Horizontal, &adjustment);
        scale.set_draw_value(false);
        scale.set_hexpand(true);
        hbox.add(&scale);

        let current_time_label = Label::new(None);
        hbox.add(&current_time_label);

        let slash_label = Label::new("/");
        hbox.add(&slash_label);

        let duration_label = Label::new(None);
        duration_label.set_margin_right(10);
        hbox.add(&duration_label);

        app_window.add(&vbox);
        app_window.show_all();

        let app = App {
            adjustment,
            cover: img,
            current_time_label,
            duration_label,
            playlist: pl,
            state,
            toolbar: mt,
            window: app_window,
        };

        app.connect_events();
        app.connect_toolbar_events();

        app
    }

    fn millis_to_minutes(millis: u64) -> String {
        let mut seconds = millis / 1_000;
        let minutes = seconds / 60;
        seconds %= 60;
        format!("{}:{:02}", minutes, seconds)
    }

    fn connect_events(&self) {
        let current_time_label = self.current_time_label.clone();
        let duration_label = self.duration_label.clone();
        let playlist = Rc::clone(&self.playlist);
        let adjustment = self.adjustment.clone();
        let state = Arc::clone(&self.state);
        // let play_image = self.toolbar.play_image.clone();
        gtk::timeout_add(100, move || {
            let state = state.lock().unwrap();
            if let Some(path) = playlist.selected_path() {
                if let Some(duration) = state.durations.get(&path) {
                    let duration = *duration;
                    adjustment.set_upper(duration as f64);
                    duration_label.set_text(&Self::millis_to_minutes(duration));
                }
            }

            current_time_label.set_text(&Self::millis_to_minutes(state.current_time));

            adjustment.set_value(state.current_time as f64);
            Continue(true)
        });
    }
}

fn main() {
    // create gio application
    let app = Application::new(
        "com.github.rust-by-example", // application id
        ApplicationFlags::empty(),
    )
    .expect("Error");

    gstreamer::init().expect("Gstreamer fails to initialize");

    app.connect_startup(|a| {
        let _ = App::new(a);
    });
    app.connect_activate(|_| {});
    app.run(&env::args().collect::<Vec<_>>());
}

fn to_millis(duration: Duration) -> u64 {
    duration.as_secs() * 1000 + duration.subsec_nanos() as u64 / 1_000_000
}
