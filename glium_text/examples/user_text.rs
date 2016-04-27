extern crate glium;
extern crate glium_text;
extern crate cgmath;

use std::path::Path;
use std::thread;
use std::time::Duration;
use glium::Surface;
use glium::glutin;

fn main() {
    use glium::DisplayBuild;
    use std::fs::File;

    let display = glutin::WindowBuilder::new().with_dimensions(1024, 768).build_glium().unwrap();
    let system = glium_text::TextSystem::new(&display);

    let font = match std::env::args().nth(1) {
        Some(file) => glium_text::FontTexture::new(&display, File::open(&Path::new(&file)).unwrap(), 70),
        None => {
            match File::open(&Path::new("C:\\Windows\\Fonts\\Arial.ttf")) {
                Ok(f) => glium_text::FontTexture::new(&display, f, 70),
                Err(_) => glium_text::FontTexture::new(&display, &include_bytes!("font.ttf")[..], 70),
            }
        }
    }.unwrap();

    let mut buffer = String::new();

    let sleep_duration = Duration::from_millis(17);

    println!("Type with your keyboard");

    'main: loop {
        let text = glium_text::TextDisplay::new(&system, &font, &buffer);

        let (w, h) = display.get_framebuffer_dimensions();

        let matrix:[[f32; 4]; 4] = cgmath::Matrix4::new(
            0.1, 0.0, 0.0, 0.0,
            0.0, 0.1 * (w as f32) / (h as f32), 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            -0.9, 0.0, 0.0, 1.0f32,
        ).into();

        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 1.0);
        glium_text::draw(&text, &system, &mut target, matrix, (1.0, 1.0, 0.0, 1.0));
        target.finish().unwrap();

        thread::sleep(sleep_duration);

        for event in display.poll_events() {
            match event {
                glutin::Event::ReceivedCharacter('\r') => buffer.clear(),
                glutin::Event::ReceivedCharacter(c) if c as u32 == 8 => { buffer.pop(); },
                glutin::Event::ReceivedCharacter(chr) => buffer.push(chr),
                glutin::Event::Closed => break 'main,
                _ => ()
            }
        }
    }
}
