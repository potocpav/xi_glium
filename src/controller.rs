
use std::time::Duration;
use std::thread;

use glium::backend::glutin_backend::GlutinFacade;
use serde_json::Value;

use core::Core;
use renderer::Renderer;
use text::Text;
use file_dialog;

// pub struct Controller {
//     core: Core,
//     display: GlutinFacade,
//     renderer: Renderer,
//
//     state: State,
// }

pub struct State {
    pub filename: Option<String>,
    pub text: Text,
    pub first_line: u64,
    pub line_count: u64,
    pub scroll_to: (u64, u64),
}

impl State {
    pub fn new(filename: Option<String>) -> State {
        State {
            filename: filename,
            text: Text::new(),
            first_line: 0,
            line_count: 1,
            scroll_to: (0, 0),
        }
    }

    // the 'data' field is specified in
    // https://github.com/google/xi-editor/blob/master/doc/frontend.md#settext
    // The line data itself is updated in fn update_lines
    pub fn update(&mut self, data: Value) {
        println!("{:?}", data);
        if let Some(array) = data.as_array() {
            if let Some("settext") = array[0].as_string() {
                if let Some(dict) = array[1].as_object() {
                    self.first_line = dict.get("first_line").unwrap().as_u64().unwrap();
                    self.line_count = dict.get("height").unwrap().as_u64().unwrap();
                    self.text.refresh(self.line_count);
                    self.text.add_lines(dict.get("lines").unwrap(), self.first_line);
                    // TODO: is this supposed to be in every message, or not?
                    if let Some(x) = dict.get("scrollto")
                                         .and_then(|x| x.as_array()) {
                        self.text.scroll_to(x[0].as_u64().unwrap(), x[1].as_u64().unwrap());
                        // self.scroll_to = (x[0].as_u64().unwrap(), x[1].as_u64().unwrap());
                    }
                }
            }
        }
    }
}

pub fn run(core_path: &str, filename: Option<String>, display: GlutinFacade) {
    let mut core = Core::new(&core_path);

    if let Some(ref filename) = filename {
        core.open(filename);
    }

    let mut state = State::new(filename);
    let renderer = Renderer::new(&display);

    // the main loop
    // TODO: replace stateful ctrl/shift modifiers by stateless ones
    let (mut ctrl, mut shift) = (false, false);
    let (mut file_open_rx, mut file_save_rx) = (None, None); // The receiver of a file dialog.
    'a: loop {
        // polling and handling the events received by the window
        for event in display.poll_events() {
            use glium::glutin::*;
            match event {
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::O)) => {
                    if ctrl && file_open_rx.is_none() {
                        file_open_rx = Some(file_dialog::open());
                        ctrl = false; // ctrl is typically released over the dialog
                    }
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                    if ctrl {
                        println!("Saving a file.");

                        if let Some(ref filename) = state.filename {
                            core.save(filename);
                        } else {
                            file_save_rx = Some(file_dialog::save());
                            ctrl = false;
                        }

                        if shift {
                            file_save_rx = Some(file_dialog::save());
                            ctrl = false;
                            shift = false;
                        }
                    }
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::T)) => {
                    if ctrl {
                        println!("Testing..");
                        println!("res: {:?}", core.render_lines_sync(0, 10));
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
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::PageUp)) => {
                    core.page_up();
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::PageDown)) => {
                    core.page_down();
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::F1)) => {
                    core.f1();
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::F2)) => {
                    core.f2();

                }, Event::KeyboardInput(state, _, Some(VirtualKeyCode::LControl))
                 | Event::KeyboardInput(state, _, Some(VirtualKeyCode::RControl)) => {
                    ctrl = state == ElementState::Pressed;
                    println!("ctrl: {}", ctrl);
                }, Event::KeyboardInput(state, _, Some(VirtualKeyCode::LShift))
                 | Event::KeyboardInput(state, _, Some(VirtualKeyCode::RShift)) => {
                    shift = state == ElementState::Pressed;
                    println!("shift: {}", shift);

                }, Event::ReceivedCharacter(ch) => {
                    if ch == '\x08' || ch == '\x7f' || ctrl {
                        continue; // delete is not implemented, backspace is special-cased, ignore ctrl-ed characters.
                    }
                    println!("ch: {:?}", ch);
                    core.char(ch);

                }, Event::Resized(w, h) => {
                    state.text.set_size(w, h);
                    core.scroll(state.text.top as u64, state.text.height.round() as u64);
                }, Event::Closed => break 'a,
                _ => ()
            }
        }

        if let Some(rx) = file_open_rx.take() {
            match rx.try_recv() {
                Ok(Some(filename)) => {
                    // TODO: replace String by Path or OsString
                    core.open(filename.to_str().unwrap());
                    state.filename = Some(filename.to_str().unwrap().into());
                    file_open_rx = None;
                }, _ => {
                    file_open_rx = Some(rx);
                }
            }
        }

        if let Some(rx) = file_save_rx.take() {
            match rx.try_recv() {
                Ok(Some(filename)) => {
                    // TODO: replace String by Path or OsString
                    core.save(filename.to_str().unwrap());
                    state.filename = Some(filename.to_str().unwrap().into());
                    file_save_rx = None;
                }, _ => {
                    file_save_rx = Some(rx);
                }
            }
        }

        while let Ok(value) = core.rx.try_recv() {
            state.update(value);
        }

        let mut target = renderer.draw(&display);

        state.text.render(&mut target).unwrap();

        // renderer.draw(&display, state.text.get_lines());

        target.finish();

        thread::sleep(Duration::from_millis(15));
    }
}
