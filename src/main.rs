
use std::thread;
use std::time::Duration;

use core::Core;
use display::Display;

mod core;
mod display;

extern crate glium;
extern crate glium_text;
extern crate serde_json;

// enum Event {
//     Update(Value),
//     Interact(glium::glutin::Event),
// }
//
// fn event_loop() {
//     thread::spawn(move || {
//
//     });
// }

fn main() {
    let filename = std::env::args().nth(1).expect("Specify filename as a first argument.");
    let executable = std::env::var("xicore").unwrap_or("../rust/target/debug/xicore".into());

    let mut core = Core::new(&executable);
    core.open(&filename);
    let mut display = Display::new();

    // the main loop
    let mut ctrl = false;
    'a: loop {
        display.draw();

        while let Ok(value) = core.rx.try_recv() {
            display.update(value);
        }

        // polling and handling the events received by the window
        for event in display.poll_events() {
            use glium::glutin::*;
            match event {
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::O)) => {
                    if ctrl {
                        println!("Opening a file..");
                        core.open(&filename);
                    }
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                    if ctrl {
                        println!("Saving a file..");
                        core.save(&filename);
                    }
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Left)) => {
                    core.left();
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Right)) => {
                    core.right();
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Up)) => {
                    core.up();
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Down)) => {
                    core.down();
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::Back)) => {
                    core.del();

                }, Event::KeyboardInput(state, _, Some(VirtualKeyCode::LControl)) => {
                    ctrl = state == ElementState::Pressed;
                    println!("ctrl: {}", ctrl);

                }, Event::ReceivedCharacter(ch) => {
                    if ch == '\x08' || ch == '\x7f' || ctrl {
                        continue; // delete is not implemented, backspace is special-cased, ignore ctrl-ed characters.
                    }
                    println!("ch: {:?}", ch);
                    core.char(ch);
                }, Event::Closed => break 'a,
                _ => ()
            }
        }

        thread::sleep(Duration::from_millis(15));
    }
}
