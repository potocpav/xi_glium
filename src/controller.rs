
use std::time::Duration;
use std::thread;

use glium::backend::glutin_backend::GlutinFacade;
use serde_json::Value;

use core::Core;
use renderer::Renderer;
use line::Line;
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
    pub text: Vec<Line>,
    pub first_line: u64,
    pub line_count: u64,
    pub scroll_to: (u64, u64),
}

#[derive(Debug)]
pub struct MouseState {
    pub last_x: i32,
    pub last_y: i32
}

impl State {
    pub fn new(filename: Option<String>) -> State {
        State {
            filename: filename,
            text: vec![],
            first_line: 0,
            line_count: 1,
            scroll_to: (0, 0),
        }
    }

    // the 'data' field is specified in
    // https://github.com/google/xi-editor/blob/master/doc/frontend.md#settext
    pub fn update(&mut self, data: Value) {
        println!("{:?}", data);
        if let Some(array) = data.as_array() {
            if let Some("settext") = array[0].as_string() {
                if let Some(dict) = array[1].as_object() {
                    let array = dict.get("lines").unwrap().as_array().unwrap();
                    self.text.clear();
                    for line in array {
                        let line = line.as_array().unwrap();
                        let text = line[0].as_string().unwrap().into();

                        // annotations
                        let mut cursor = None;
                        for annotation in line.iter().skip(1).map(|a| a.as_array().unwrap()) {
                            match annotation[0].as_string().unwrap() {
                                "cursor" => {
                                    cursor = Some(annotation[1].as_u64().unwrap());
                                },
                                _ => () // ignore unknown annotations
                            }

                        }

                        self.text.push(Line { text: text, cursor: cursor });
                    }

                    self.first_line = dict.get("first_line").unwrap().as_u64().unwrap();
                    self.line_count = dict.get("height").unwrap().as_u64().unwrap();
                    // TODO: check if it is supposed to be in every message or not
                    if let Some(x) = dict.get("scrollto")
                                         .and_then(|x| x.as_array()) {
                        self.scroll_to = (x[0].as_u64().unwrap(), x[1].as_u64().unwrap());
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

    let mut lines_y: Option<Vec<i32>> = None;

    // the main loop
    let (mut ctrl, mut shift) = (false, false);
    let (mut file_open_rx, mut file_save_rx) = (None, None); // The receiver of a file dialog.
    let mut mouse_info = MouseState { last_x: 0, last_y: 0 };
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
                        println!("Saving a file..");

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
                        core.test();
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

                },
                Event::MouseMoved(mouse_x, mouse_y) => {
                    mouse_info.last_x = mouse_x;
                    mouse_info.last_y = mouse_y;
                }
                Event::MouseInput(button_state, button) => {
                    if button_state != ElementState::Released || button != MouseButton::Left {
                        continue;
                    }
                    match &lines_y {
                        &Some(ref ly) => {
                            match find_line(ly, mouse_info.last_y) {
                                Some(line) => {
                                    let pos = renderer.find_colum(&state, mouse_info.last_x, line);
                                    core.click(pos);
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }

                },
                Event::ReceivedCharacter(ch) => {
                    if ch == '\x08' || ch == '\x7f' || ctrl {
                        continue; // delete is not implemented, backspace is special-cased, ignore ctrl-ed characters.
                    }
                    println!("ch: {:?}", ch);
                    core.char(ch);

                }, Event::Resized(w,h) => {
                    let ly = get_lines_y(h);
                    core.scroll(state.first_line, ly.len() as u64);
                    lines_y = Some(ly);
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

        if let Some(ref lines_y) = lines_y {
            renderer.draw(&display, &state, lines_y);
        }

        thread::sleep(Duration::from_millis(15));
    }
}

fn find_line(lines: &Vec<i32>, pos: i32) -> Option<usize> {
    lines.iter().rev().position(|&y| y >= pos)
}

fn get_lines_y(height: u32) -> Vec<i32> {
    let line_h = 20;
    let margin = 15;

    (0..).map(|i| height as i32 - margin - line_h/2 - i * line_h)
         .take_while(|i| *i >= margin + line_h/2)
         .collect()
}
