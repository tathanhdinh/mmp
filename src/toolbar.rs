use std::{path::PathBuf, rc::Rc, sync::Arc};

use gtk::{
    ApplicationWindow, ContainerExt, DialogExt, FileChooserAction, FileChooserDialog,
    FileChooserExt, FileFilter, FileFilterExt, Image, ImageExt, SeparatorToolItem, ToolButton,
    ToolButtonExt, Toolbar, WidgetExt,
};
use gtk_sys::{
    GTK_RESPONSE_ACCEPT, GTK_RESPONSE_CANCEL, GTK_STOCK_MEDIA_PAUSE, GTK_STOCK_MEDIA_PLAY,
    GTK_STOCK_OK,
};

use libc::c_char;

use crate::playlist::Playlist;

const PLAY_STOCK: &'static str = "gtk-media-play";
const PAUSE_STOCK: &'static str = "gtk-media-pause";

pub(crate) const PLAY_ICON: &'static str = PLAY_STOCK;
pub(crate) const PAUSE_ICON: &'static str = PAUSE_STOCK;

pub(crate) fn set_image_icon(button: &ToolButton, icon: *const c_char) {
    button.set_stock_id(icon)
}

pub(crate) struct MusicToolbar {
    pub open_button: ToolButton,
    pub next_button: ToolButton,
    pub play_button: ToolButton,
    pub previous_button: ToolButton,
    pub quit_button: ToolButton,
    pub remove_button: ToolButton,
    pub stop_button: ToolButton,
    pub toolbar: Toolbar,
}

impl MusicToolbar {
    pub fn new() -> Self {
        let toolbar = Toolbar::new();

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

        MusicToolbar {
            open_button,
            next_button,
            play_button,
            previous_button,
            quit_button,
            remove_button,
            stop_button,
            toolbar,
        }
    }
}

use crate::App;

impl App {
    const RESPONSE_ACCEPT: i32 = GTK_RESPONSE_ACCEPT as i32;
    const RESPONSE_CANCEL: i32 = GTK_RESPONSE_CANCEL as i32;

    fn show_open_dialog(parent: &ApplicationWindow) -> Option<PathBuf> {
        let dialog = FileChooserDialog::new(
            Some("Select an MP3 audio file"),
            Some(parent),
            FileChooserAction::Open,
        );
        let filter = FileFilter::new();
        filter.add_mime_type("audio/mp3");
        filter.set_name("MP3 audio file");
        dialog.add_filter(&filter);
        dialog.add_button("Cancel", Self::RESPONSE_CANCEL);
        dialog.add_button("Accept", Self::RESPONSE_ACCEPT);

        if dialog.run() == Self::RESPONSE_ACCEPT {
            let file = dialog.get_filename();
            dialog.destroy();
            file
        } else {
            None
        }
    }

    pub fn connect_toolbar_events(&self) {
        let window = self.window.clone();
        self.toolbar.quit_button.connect_clicked(move |_| {
            window.destroy();
        });

        // let playlist = self.playlist.clone();
        let playlist = Rc::clone(&self.playlist);
        let cover = self.cover.clone();
        let state = Arc::clone(&self.state);

        let play_button = self.toolbar.play_button.clone();

        self.toolbar.play_button.connect_clicked(move |_| {
            if state.lock().unwrap().stopped {
                if playlist.play() {
                    play_button.set_stock_id(PAUSE_STOCK);
                    Self::set_cover(&cover, &playlist);
                } else {
                    // play_button.set_stock_id(PLAY_STOCK);
                    play_button.set_icon_name(GTK_STOCK_MEDIA_PLAY);
                }
            }
            // if play_button.get_stock_id() == Some(String::from(PLAY_STOCK)) {
            //     play_button.set_stock_id(PAUSE_STOCK);
            //     Self::set_cover(&cover, &playlist);
            // } else {
            //     play_button.set_stock_id(PLAY_STOCK);
            // }
        });

        let parent = self.window.clone();
        let playlist = Rc::clone(&self.playlist);
        self.toolbar.open_button.connect_clicked(move |_| {
            let file = Self::show_open_dialog(&parent);
            if let Some(file) = file {
                playlist.add(&file);
            }
        });

        let playlist = Rc::clone(&self.playlist);
        self.toolbar.remove_button.connect_clicked(move |_| {
            playlist.remove_selection();
        });
    }

    fn set_cover(cover: &Image, playlist: &Rc<Playlist>) {
        cover.set_from_pixbuf(playlist.pixbuf().as_ref());
        cover.show();
    }

    // fn set_image_icon(button: &ToolButton, )
}
