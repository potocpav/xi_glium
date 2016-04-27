
use std::path::Path;
use std::fs::File;

use glium;
use glium_text;
use glium::Surface;

use serde_json::Value;

struct Line {
    text: String,
    cursor: Option<u64>,
}

pub struct Display {
    display:  glium::backend::glutin_backend::GlutinFacade,
    text_system: glium_text::TextSystem,
    font_texture: glium_text::FontTexture,
    text: Vec<Line>,
}

impl Display {
    pub fn new() -> Display {
        let font_size = 15;

        use glium::DisplayBuild;
        let display = glium::glutin::WindowBuilder::new()
            .with_dimensions(760, 380)
            .with_title(format!("xi_glium"))
            .build_glium()
            .unwrap();

        let text_system = glium_text::TextSystem::new(&display);
        let font_texture = glium_text::FontTexture::new(&display, File::open(&Path::new("Hack-Regular.ttf")).unwrap(), font_size).unwrap();

        Display {
            display: display,
            text_system: text_system,
            font_texture: font_texture,
            text: vec![],
        }
    }

    pub fn draw(&mut self) {
        let mut target = self.display.draw();
        target.clear_color(1.0, 1.0, 1.0, 0.0);

        self.draw_text(&mut target).unwrap();

        target.finish().unwrap();
    }

    pub fn update(&mut self, data: Value) {
        println!("Updating to {:?}...", data);
        if let Some(array) = data.as_array() {
            if let Some("settext") = array[0].as_string() {
                if let Some(dict) = array[1].as_object() {
                    let array = dict.get("lines").unwrap().as_array().unwrap();
                    self.text.clear();
                    for line in array {
                        let line = line.as_array().unwrap();
                        let cursor = if line.len() > 1 { // Cursor is present
                                let cursor = line[1].as_array().unwrap();
                                assert!(cursor[0].as_string().unwrap() == "cursor");
                                Some(cursor[1].as_u64().unwrap())
                            } else {
                                None
                            };
                        self.text.push(Line { text: line[0].as_string().unwrap().into(), cursor: cursor });
                        // println!("line: {:?}", line);
                    }
                }
            }
        }
    }

    fn draw_text(&self, target: &mut glium::Frame)
            -> Result<(), glium::DrawError> {
        let (w, h) = target.get_dimensions();
        let line_h = 30;
        let margin = 15;
        for (i,line) in self.text.iter().enumerate() {
            try!(self.draw_line(target, line, (15, h as i32 - margin - line_h/2 - i as i32 * line_h)));
        }
        Ok(())
    }

    fn draw_line(&self, target: &mut glium::Frame, line: &Line, (px, py): (i32, i32))
            -> Result<(), glium::DrawError> {
        let size = self.font_texture.em_pixels();
        let (w, h) = target.get_dimensions();
        let text_tf = |px: i32, py: i32| -> [[f32; 4]; 4] {
            let (x, y) = (px as f32 / w as f32 * 2. - 1.,
                         (py as f32 - size as f32 / 2.) / h as f32 * 2. - 1.);

            let scale = 2. * size as f32;

            [[scale / w as f32, 0.0, 0.0, 0.0],
             [0.0, scale / h as f32, 0.0, 0.0],
             [0.0, 0.0, 1.0, 0.0],
             [x, y, 0.0, 1.0]]
        };

        let text = glium_text::TextDisplay::new(&self.text_system, &self.font_texture, &line.text);

        glium_text::draw(&text, &self.text_system, target, text_tf(px, py), (0., 0., 0., 1.));

        if let Some(mut pos) = line.cursor {
            let cursor = glium_text::TextDisplay::new(&self.text_system, &self.font_texture, "|");
            // println!("line length: {}", line.text.len());
            if pos >= text.get_char_pos_x().len() as u64 {
                pos = (text.get_char_pos_x().len() - 1) as u64;
            }
            let offset_local = text.get_char_pos_x()[pos as usize] - cursor.get_char_pos_x()[1] / 2.;
            let offset_screen = (offset_local * size as f32) as i32;

            // println!("Drawing cursor on {}th char = {}...", pos, offset_screen);

            glium_text::draw(&cursor, &self.text_system, target, text_tf(px + offset_screen, py), (0., 0., 1., 1.));
        }
        Ok(())
    }

    pub fn poll_events(&self) -> glium::backend::glutin_backend::PollEventsIter { self.display.poll_events() }
}
