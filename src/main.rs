mod mp3;
mod player;
mod playlist;
mod toolbar;

use self::{playlist::Playlist, toolbar::MusicToolbar};

use gio::{ApplicationExt, ApplicationExtManual, ApplicationFlags};
use gtk::{
    prelude::Inhibit,
    Adjustment, Application, ApplicationWindow, ApplicationWindowExt, ContainerExt, GtkWindowExt,
    Image, ImageExt,
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
    playlist: Rc<Playlist>,
    state: Arc<Mutex<State>>,
    toolbar: MusicToolbar,
    window: ApplicationWindow,
}

impl App {
    fn new(app: &Application) -> Self {
        let aw = ApplicationWindow::new(app);
        aw.set_title("A minimal music player");

        let mt = MusicToolbar::new();
        let vbox = gtk::Box::new(Vertical, 0);
        vbox.add(&mt.toolbar);

        let state = Arc::new(Mutex::new(State { stopped: true }));

        // add playlist
        let pl = Rc::new(Playlist::new(state.clone()));
        vbox.add(&pl.treeview);

        // add cover...
        let img = Image::new();
        vbox.add(&img);

        // add scale
        let adj = Adjustment::new(0.0, 0.0, 10.0, 0.0, 0.0, 0.0);
        let scale = Scale::new(Horizontal, &adj);
        scale.set_draw_value(false);
        vbox.add(&scale);
        aw.add(&vbox);

        aw.show_all();

        let app = App {
            adjustment: adj,
            cover: img,
            playlist: pl,
            state,
            toolbar: mt,
            window: aw,
        };

        app.connect_events();
        app.connect_toolbar_events();

        app
    }

    fn connect_events(&self) {}
}

fn main() {
    // create gio application
    let app = Application::new(
        "com.github.rust-by-example", // application id
        ApplicationFlags::empty(),
    )
    .expect("Error");

    app.connect_startup(|a| {
        let _ = App::new(a);
    });
    app.connect_activate(|_| {});
    app.run(&env::args().collect::<Vec<_>>());
}

fn to_millis(duration: Duration) -> u64 {
    duration.as_secs() * 1000 + duration.subsec_nanos() as u64 / 1_000_000
}
