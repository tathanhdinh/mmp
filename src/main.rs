mod toolbar;

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

const PLAY_STOCK: &'static str = "gtk-media-play";

struct App {
    toolbar: MusicToolbar,
    window: ApplicationWindow,
}

impl App {
    fn new(app: Application) -> Self {
        let aw = ApplicationWindow::new(&app);
        aw.set_title("mmp");

        let mt = MusicToolbar::new();
        // aw.add(&mt.toolbar);
        let vbox = gtk::Box::new(Vertical, 0);
        vbox.add(&mt.toolbar);

        let adj = Adjustment::new(0.0, 0.0, 10.0, 0.0, 0.0, 0.0);
        let scale = Scale::new(Horizontal, &adj);
        scale.set_draw_value(false);
        vbox.add(&scale);

        aw.add(&vbox);
        aw.show_all();

        let app = App {
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
        // let aw = ApplicationWindow::new(&a);
        let aw = ApplicationWindow::new(a);
        aw.set_title("mmp");
        // aw.connect_delete_event(|_, _| Inhibit(true));
        // aw.show();

        let toolbar = Toolbar::new();
        aw.add(&toolbar);

        let open_button = ToolButton::new_from_stock("gtk-open");
        toolbar.add(&open_button);

        toolbar.add(&SeparatorToolItem::new());

        let previous_button = ToolButton::new_from_stock("gtk-media-previous");
        toolbar.add(&previous_button);

        let play_button = ToolButton::new_from_stock(PLAY_STOCK);
        toolbar.add(&play_button);

        let stop_button = ToolButton::new_from_stock("gtk-media-stop");
        toolbar.add(&stop_button);

        let next_button = ToolButton::new_from_stock("gtk-media-next");
        toolbar.add(&next_button);

        toolbar.add(&SeparatorToolItem::new());

        let remove_button = ToolButton::new_from_stock("gtk-remove");
        toolbar.add(&remove_button);

        toolbar.add(&SeparatorToolItem::new());

        let quit_button = ToolButton::new_from_stock("gtk-quit");
        toolbar.add(&quit_button);

        aw.show_all();
    });
    app.connect_activate(|_| {});
    app.run(&env::args().collect::<Vec<_>>());
}
