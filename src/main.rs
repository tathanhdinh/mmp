mod playlist;
mod toolbar;
mod mp3;

use std::time::Duration;
use self::playlist::Playlist;
use self::toolbar::MusicToolbar;

use gio::{ApplicationExt, ApplicationExtManual, ApplicationFlags};
use gtk::prelude::Inhibit;
use gtk::{
    Adjustment, Application, ApplicationWindow, ApplicationWindowExt, ContainerExt, GtkWindowExt,
    Image, ImageExt,
    Orientation::{Horizontal, Vertical},
    Scale, ScaleExt, SeparatorToolItem, ToolButton, ToolButtonExt, Toolbar, WidgetExt,
};
use std::env;
use std::rc::Rc;

const PLAY_STOCK: &'static str = "gtk-media-play";

struct App {
    adjustment: Adjustment,
    cover: Image,
    playlist: Rc<Playlist>,
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

        // add playlist
        let pl = Rc::new(Playlist::new());
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
