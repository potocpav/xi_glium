

mod core;
mod renderer;
mod controller;
mod line;
// mod file_dialog;

extern crate glium;
extern crate glium_text;
extern crate serde_json;

fn main() {
    let filename = std::env::args().nth(1).expect("Specify filename as a first argument.");
    let core_path = std::env::var("xicore").unwrap_or("../xi-editor/rust/target/debug/xicore".into());

    use glium::DisplayBuild;
    let display = glium::glutin::WindowBuilder::new()
        .with_dimensions(760, 380)
        .with_title(format!("xi_glium"))
        .build_glium()
        .unwrap();

    controller::run(&core_path, filename, display);
}
