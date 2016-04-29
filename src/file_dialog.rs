
use std::sync::mpsc;
use std::path;
use gtk;
use gtk::prelude::*;
use glib;


pub fn open() -> mpsc::Receiver<Option<path::PathBuf>> {
    spawn(Action::Open)
}

pub fn save() -> mpsc::Receiver<Option<path::PathBuf>> {
    spawn(Action::Save)
}

enum Action {
    Open,
    Save,
}

fn spawn(action: Action) -> mpsc::Receiver<Option<path::PathBuf>> {
    let (sx, rx) = mpsc::sync_channel(0);
    glib::idle_add(move || {
        let file_chooser = match action {
            Action::Open => {
                let fc = gtk::FileChooserDialog::new(
                    Some("Open File"), None::<&gtk::Window>, gtk::FileChooserAction::Open);
                fc.add_buttons(&[
                    ("Open", gtk::ResponseType::Ok as i32),
                    ("Cancel", gtk::ResponseType::Cancel as i32),
                    ]);
                fc
            }, Action::Save => {
                let fc = gtk::FileChooserDialog::new(
                    Some("Save File"), None::<&gtk::Window>, gtk::FileChooserAction::Save);
                fc.add_buttons(&[
                    ("Save", gtk::ResponseType::Ok as i32),
                    ("Cancel", gtk::ResponseType::Cancel as i32),
                    ]);
                fc
            }
        };

        if file_chooser.run() == gtk::ResponseType::Ok as i32 {
            let filename = file_chooser.get_filename().unwrap();
            sx.send(Some(filename)).unwrap();
        } else {
            sx.send(None).unwrap();
        }
        file_chooser.destroy();

        glib::Continue(false)
    });
    rx
}
