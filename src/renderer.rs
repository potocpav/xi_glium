
use std::path::Path;
use std::fs::File;

use glium;
use glium_text;
use glium::Surface;

use line::Line;
use controller::State;


pub struct Renderer {
    text_system: glium_text::TextSystem,
    font_texture: glium_text::FontTexture,
}

impl Renderer {
    pub fn new(display: &glium::backend::glutin_backend::GlutinFacade) -> Renderer {
        let font_size = 15;

        let text_system = glium_text::TextSystem::new(display);
        let font_texture = glium_text::FontTexture::new(display, File::open(&Path::new("Hack-Regular.ttf")).unwrap(), font_size).unwrap();

        Renderer {
            // display: display,
            text_system: text_system,
            font_texture: font_texture,
        }
    }

    pub fn draw(&self, display: &glium::backend::glutin_backend::GlutinFacade, state: &State, lines_y: &[i32]) {
        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 0.0);

        self.draw_text(&mut target, state, lines_y).unwrap();

        target.finish().unwrap();
    }

    fn draw_text(&self, target: &mut glium::Frame, state: &State, lines_y: &[i32])
            -> Result<(), glium::DrawError> {
        let start = ::std::cmp::max(state.scroll_to.0 as i64 - state.first_line as i64 - lines_y.len() as i64 + 1, 0);
        for (i,(line,y)) in state.text.iter()
                                      .skip(start as usize)
                                      .zip(lines_y.iter())
                                      .enumerate() {
            try!(self.draw_line(target, line, (15, *y)));
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
            if pos >= text.get_char_pos_x().len() as u64 {
                pos = (text.get_char_pos_x().len() - 1) as u64;
            }
            let offset_local = text.get_char_pos_x()[pos as usize] - cursor.get_char_pos_x()[1] / 2.;
            let offset_screen = (offset_local * size as f32) as i32;

            glium_text::draw(&cursor, &self.text_system, target, text_tf(px + offset_screen, py), (0., 0., 1., 1.));
        }
        Ok(())
    }

    // pub fn poll_events(&self) -> glium::backend::glutin_backend::PollEventsIter { self.display.poll_events() }
}
