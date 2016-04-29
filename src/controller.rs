
use std::time::Duration;
use std::thread;

use glium::backend::glutin_backend::GlutinFacade;
use serde_json::Value;


use core::Core;
use renderer::Renderer;
use line::Line;

// pub struct Controller {
//     core: Core,
//     display: GlutinFacade,
//     renderer: Renderer,
//
//     state: State,
// }

pub struct State {
    pub filename: String,
    pub text: Vec<Line>,
    pub first_line: u64,
    pub line_count: u64,
    pub scroll_to: (u64, u64),
}

impl State {
    pub fn new(filename: String) -> State {
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

pub fn run(core_path: &str, filename: String, display: GlutinFacade) {
    let mut core = Core::new(&core_path);
    core.open(&filename);

    let mut state = State::new(filename);

    let renderer = Renderer::new(&display);

    let mut lines_y: Option<Vec<i32>> = None;

    // the main loop
    let mut ctrl = false;
    'a: loop {
        if let Some(ref lines_y) = lines_y {
            renderer.draw(&display, &state, lines_y);
        }

        while let Ok(value) = core.rx.try_recv() {
            state.update(value);
        }

        // polling and handling the events received by the window
        for event in display.poll_events() {
            use glium::glutin::*;
            match event {
                Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::O)) => {
                    if ctrl {
                        println!("Opening a file..");
                        // dialog.spawn();
                        core.open(&state.filename);
                    }
                }, Event::KeyboardInput(ElementState::Pressed, _, Some(VirtualKeyCode::S)) => {
                    if ctrl {
                        println!("Saving a file..");
                        core.save(&state.filename);
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

                }, Event::ReceivedCharacter(ch) => {
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

        thread::sleep(Duration::from_millis(15));
    }
}

fn get_lines_y(height: u32) -> Vec<i32> {
    let line_h = 20;
    let margin = 15;

    (0..).map(|i| height as i32 - margin - line_h/2 - i * line_h)
         .take_while(|i| *i >= margin + line_h/2)
         .collect()
}
