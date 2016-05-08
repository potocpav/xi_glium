
mod core;
mod renderer;
mod controller;
mod text;
mod file_dialog;

#[macro_use]
extern crate glium;
extern crate glium_text;
extern crate serde_json;
extern crate gtk;
extern crate glib; // Needed by gtk to supply a threaded fn idle_add
extern crate clipboard;

fn main() {
    let filename = std::env::args().nth(1);
    let core_path = std::env::var("xicore").unwrap_or("../xi-editor/rust/target/debug/xicore".into());

    // I read that GTK on Mac needs to be in the main thread. We must let it have it.
    ::std::thread::spawn(move || {
        use glium::DisplayBuild;
        let display = glium::glutin::WindowBuilder::new()
            .with_dimensions(760, 380)
            .with_title(String::from("xi_glium"))
            .build_glium()
            .unwrap();
        display.get_window().unwrap().set_cursor(glium::glutin::MouseCursor::Text);

        controller::run(&core_path, filename, display);

        glib::idle_add(|| { gtk::main_quit(); glib::Continue(false) });
    });

    gtk::init().expect("Failed to initialize GTK.");
    gtk::main();
}
