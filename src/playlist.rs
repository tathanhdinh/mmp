use std::{
    cell::RefCell,
    path::Path,
    sync::{Arc, Mutex},
    thread,
};

use gdk_pixbuf::{InterpType, Pixbuf, PixbufExt, PixbufLoader, PixbufLoaderExt};
use gtk::{
    CellLayoutExt, CellRendererPixbuf, CellRendererText, GtkListStoreExt, GtkListStoreExtManual,
    ListStore, StaticType, ToValue, TreeIter, TreeModelExt, TreeSelectionExt, TreeView,
    TreeViewColumn, TreeViewColumnExt, TreeViewExt, Type, WidgetExt,
};
use id3::Tag;

use crate::{player::Player, State};

const THUMBNAIL_COLUMN: u32 = 0;
const TITLE_COLUMN: u32 = 1;
const ARTIST_COLUMN: u32 = 2;
const ALBUM_COLUMN: u32 = 3;
const GENRE_COLUMN: u32 = 4;
const YEAR_COLUMN: u32 = 5;
const TRACK_COLUMN: u32 = 6;
const PATH_COLUMN: u32 = 7;
const PIXBUF_COLUMN: u32 = 8;

const IMAGE_SIZE: i32 = 256;
const THUMBNAIL_SIZE: i32 = 64;

pub(crate) struct Playlist {
    current_song: RefCell<Option<String>>,
    pub model: ListStore,
    player: Player,
    state: Arc<Mutex<State>>,
    pub treeview: TreeView,
}

#[derive(PartialEq)]
enum Visibility {
    Invisible,
    Visisble,
}

impl Playlist {
    fn add_text_column(tw: &TreeView, title: &str, col: i32) {
        // create column then set title
        let tvc = TreeViewColumn::new();
        tvc.set_title(title);

        let crt = CellRendererText::new();

        tvc.set_expand(true);
        tvc.pack_start(&crt, true);
        tvc.add_attribute(&crt, "text", col);

        tw.append_column(&tvc);
    }

    fn add_pixbuf_column(tw: &TreeView, col: i32, vis: Visibility) {
        let tvc = TreeViewColumn::new();
        if vis == Visibility::Visisble {
            let crp = CellRendererPixbuf::new();

            tvc.pack_start(&crp, true);
            tvc.add_attribute(&crp, "pixbuf", col);
        }
        tw.append_column(&tvc);
    }

    fn create_columns(tw: &TreeView) {
        use self::Visibility::*;
        Self::add_pixbuf_column(tw, THUMBNAIL_COLUMN as i32, Visisble);
        Self::add_text_column(tw, "Title", TITLE_COLUMN as i32);
        Self::add_text_column(tw, "Artist", ARTIST_COLUMN as i32);
        Self::add_text_column(tw, "Album", ALBUM_COLUMN as i32);
        Self::add_text_column(tw, "Genre", GENRE_COLUMN as i32);
        Self::add_text_column(tw, "Year", YEAR_COLUMN as i32);
        Self::add_text_column(tw, "Track", TRACK_COLUMN as i32);
        Self::add_pixbuf_column(tw, PIXBUF_COLUMN as i32, Invisible);
    }

    const INTERP_HYPER: InterpType = InterpType::Bilinear;

    fn set_pixbuf(&self, row: &TreeIter, tag: &Tag) {
        if let Some(pict) = tag.pictures().next() {
            let pbl = PixbufLoader::new();
            pbl.set_size(IMAGE_SIZE, IMAGE_SIZE);
            pbl.write(&pict.data).unwrap();

            if let Some(pb) = pbl.get_pixbuf() {
                let tbn = pb.scale_simple(THUMBNAIL_SIZE, THUMBNAIL_SIZE, Self::INTERP_HYPER);
                self.model.set_value(row, THUMBNAIL_COLUMN, &tbn.to_value());
                self.model.set_value(row, PIXBUF_COLUMN, &pb.to_value());
            }

            pbl.close().unwrap();
        }
    }

    pub(crate) fn new(state: Arc<Mutex<State>>) -> Self {
        let model = ListStore::new(&[
            Pixbuf::static_type(),
            Type::String,
            Type::String,
            Type::String,
            Type::String,
            Type::String,
            Type::String,
            Type::String,
            Pixbuf::static_type(),
        ]);
        let tw = TreeView::new_with_model(&model);
        tw.set_hexpand(true);
        tw.set_vexpand(true);

        Self::create_columns(&tw);

        Playlist {
            current_song: RefCell::new(None),
            model,
            player: Player::new(state.clone()),
            state,
            treeview: tw,
        }
    }

    fn compute_duration(&self, path: &Path) {
        let state = Arc::clone(&self.state);
        let path = path.to_string_lossy().to_string();
        thread::spawn(move || {
            if let Some(duration) = Player::compute_duration(&path) {
                let mut state = state.lock().unwrap();
                state.durations.insert(path, crate::to_millis(duration));
            }
        });
    }

    pub(crate) fn add(&self, path: &Path) {
        self.compute_duration(path);

        let filename = path
            .file_stem()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();

        let row = self.model.append();

        if let Ok(tag) = Tag::read_from_path(path) {
            let title = tag.title().unwrap_or(filename);
            let artist = tag.artist().unwrap_or("unknown");
            let album = tag.album().unwrap_or("unknown");
            let genre = tag.genre().unwrap_or("unknown");
            let year = tag
                .year()
                .map(|y| y.to_string())
                .unwrap_or("unknown".to_string());
            let track = tag
                .track()
                .map(|tr| tr.to_string())
                .unwrap_or("unknown".to_string());
            let total_tracks = tag
                .total_tracks()
                .map(|ttr| ttr.to_string())
                .unwrap_or("unknown".to_string());
            let tr_val = format!("{} / {}", track, total_tracks);

            self.set_pixbuf(&row, &tag);

            self.model.set_value(&row, TITLE_COLUMN, &title.to_value());
            self.model
                .set_value(&row, ARTIST_COLUMN, &artist.to_value());
            self.model.set_value(&row, ALBUM_COLUMN, &album.to_value());
            self.model.set_value(&row, GENRE_COLUMN, &genre.to_value());
            self.model.set_value(&row, YEAR_COLUMN, &year.to_value());
            self.model.set_value(&row, TRACK_COLUMN, &tr_val.to_value());
        } else {
            self.model
                .set_value(&row, TITLE_COLUMN, &filename.to_value());
        }

        let path = path.to_str().unwrap_or_default();
        self.model.set_value(&row, PATH_COLUMN, &path.to_value());
    }

    pub(crate) fn remove_selection(&self) {
        let selection = self.treeview.get_selection();
        if let Some((_, iter)) = selection.get_selected() {
            self.model.remove(&iter);
        }
    }

    pub(crate) fn pixbuf(&self) -> Option<Pixbuf> {
        let selection = self.treeview.get_selection();
        if let Some((_, iter)) = selection.get_selected() {
            let value = self.model.get_value(&iter, PIXBUF_COLUMN as i32);
            value.get()
        } else {
            None
        }
    }

    pub fn selected_path(&self) -> Option<String> {
        let selection = self.treeview.get_selection();
        if let Some((_, iter)) = selection.get_selected() {
            let value = self.model.get_value(&iter, PATH_COLUMN as i32);
            return value.get::<String>();
        }
        None
    }

    pub fn play(&self) -> bool {
        if let Some(path) = self.selected_path() {
            self.player.load(&path);
            println!("start playing {}", path);
            true
        } else {
            false
        }
    }
}
