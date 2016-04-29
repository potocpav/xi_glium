
use std::path::Path;
use std::fs::File;

use glium;
use glium_text;
use glium::Surface;
use glium::index::PrimitiveType;

use line::Line;
use controller::State;


pub struct Renderer {
    program: glium::Program,
    text_system: glium_text::TextSystem,
    font_texture: glium_text::FontTexture,
}

impl Renderer {
    pub fn new(display: &glium::backend::glutin_backend::GlutinFacade) -> Renderer {
        let font_size = 15;

        let text_system = glium_text::TextSystem::new(display);
        let font_texture = glium_text::FontTexture::new(display, File::open(&Path::new("Hack-Regular.ttf")).unwrap(), font_size).unwrap();

        let program = {
            let vs_src = r#"
                #version 140

                in vec2 position;
                in vec4 color;
                out vec4 v_color;

                uniform vec2 win_size;

                void main() {
                    v_color = color;
                    gl_Position = vec4(position / win_size * 2. - 1., 0.0, 1.0);
                }
            "#;
            let fs_src = r#"
                #version 140

                in vec4 v_color;
                out vec4 color;

                void main() {
                    color = v_color;
                }
            "#;
            glium::Program::from_source(display, vs_src, fs_src, None).unwrap()
        };

        Renderer {
            program: program,
            text_system: text_system,
            font_texture: font_texture,
        }
    }

    pub fn draw(&self, display: &glium::backend::glutin_backend::GlutinFacade, state: &State, lines_y: &[i32]) {
        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 0.0);
        let (w, h) = target.get_dimensions();

        let frame = Primitive::new_rect(display, (0., h as f32), (w as f32, 0.), [0.8, 0.8, 0.8, 1.0]);
        let bg = Primitive::new_rect(display, (15., h as f32 - 15.), (w as f32 - 15., 15.), [1.0, 1.0, 1.0, 1.0]);
        frame.draw(&mut target, &self, true);
        bg.draw(&mut target, &self, true);

        self.draw_text(&mut target, state, lines_y).unwrap();

        target.finish().unwrap();
    }

    fn draw_minimap(&self) -> Result<(), glium::DrawError> {
        unimplemented!()
    }

    fn draw_text(&self, target: &mut glium::Frame, state: &State, lines_y: &[i32])
            -> Result<(), glium::DrawError> {
        let start = ::std::cmp::max(state.scroll_to.0 as i64 - state.first_line as i64 - lines_y.len() as i64 + 1, 0);
        for (i,(line,y)) in state.text.iter()
                                      .skip(start as usize)
                                      .zip(lines_y.iter())
                                      .enumerate() {
            try!(self.draw_line(target, line, (15., *y as f32), i as u64 + state.first_line));
        }
        Ok(())
    }

    fn draw_line(&self, target: &mut glium::Frame, line: &Line, (px, py): (f32, f32), line_nr: u64)
            -> Result<(), glium::DrawError> {
        let size = self.font_texture.em_pixels();
        let (w, h) = target.get_dimensions();
        let text_tf = |px: f32, py: f32| -> [[f32; 4]; 4] {
            let (x, y) = (px / w as f32 * 2. - 1.,
                         (py - size as f32 / 2.) / h as f32 * 2. - 1.);

            let scale = 2. * size as f32;

            [[scale / w as f32, 0.0, 0.0, 0.0],
             [0.0, scale / h as f32, 0.0, 0.0],
             [0.0,              0.0, 1.0, 0.0],
             [  x,                y, 0.0, 1.0]]
        };

        let text = glium_text::TextDisplay::new(&self.text_system, &self.font_texture, &line.text);

        glium_text::draw(&text, &self.text_system, target, text_tf(px, py), (0., 0., 0., 1.));

        if let Some(mut pos) = line.cursor {
            let cursor = glium_text::TextDisplay::new(&self.text_system, &self.font_texture, "|");
            if pos >= text.get_char_pos_x().len() as u64 {
                pos = (text.get_char_pos_x().len() - 1) as u64;
            }
            let offset_local = text.get_char_pos_x()[pos as usize] - cursor.get_char_pos_x()[1] / 2.;
            let offset_screen = offset_local * size as f32;

            glium_text::draw(&cursor, &self.text_system, target, text_tf(px + offset_screen, py), (0., 0., 1., 1.));
        }
        Ok(())
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
}
implement_vertex!(Vertex, position, color);

pub struct Primitive {
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer:  glium::index::NoIndices,
}

impl Primitive {
    pub fn new(display: &glium::Display, verts: &[Vertex], primitive_type: glium::index::PrimitiveType) -> Self {
        Primitive {
            vertex_buffer: glium::VertexBuffer::new(display, verts).unwrap(),
            index_buffer: glium::index::NoIndices(primitive_type),
        }
    }

    pub fn new_rect(display: &glium::Display, p1: (f32,f32), p2: (f32,f32), color: [f32; 4]) -> Self {
        let verts = vec![
            Vertex { position: [p1.0, p1.1], color: color },
            Vertex { position: [p2.0, p1.1], color: color },
            Vertex { position: [p1.0, p2.1], color: color },
            Vertex { position: [p2.0, p2.1], color: color },
        ];
        Primitive {
            vertex_buffer: glium::VertexBuffer::new(display, &verts).unwrap(),
            index_buffer:  glium::index::NoIndices(PrimitiveType::TriangleStrip),
        }
    }

    // /// Draw a shaded rectangle.
    // pub fn new_rect(display: &glium::Display, rect: Rect, colors: [[f32; 4]; 2]) -> Self {
    //     let (x1, x2, y1, y2) = (rect.left, rect.right(), rect.bottom, rect.top());
    //     let mut verts = Vec::with_capacity(GRADIENT_STEPS as usize * 2 + 2);
    //     for i in 0..GRADIENT_STEPS+1 {
    //         let interp = i as f32 / GRADIENT_STEPS as f32;
    //         let color = [colors[0][0] * (1.-interp) + colors[1][0] * interp,
    //                      colors[0][1] * (1.-interp) + colors[1][1] * interp,
    //                      colors[0][2] * (1.-interp) + colors[1][2] * interp,
    //                      colors[0][3] * (1.-interp) + colors[1][3] * interp];
    //         let posy = y1 as f32 * (1.-interp) + y2 as f32 * interp;
    //         verts.push(Vertex { position: [x1 as f32, posy], color: color });
    //         verts.push(Vertex { position: [x2 as f32, posy], color: color });
    //     }
    //     Primitive {
    //         vertex_buffer: glium::VertexBuffer::new(display, &verts).unwrap(),
    //         index_buffer: glium::index::NoIndices(PrimitiveType::TriangleStrip),
    //     }
    // }

    pub fn draw(&self, target: &mut glium::Frame, renderer: &Renderer, fill: bool) -> Result<(), glium::DrawError> {
        let (w, h) = target.get_dimensions();
        let params = glium::DrawParameters {
            polygon_mode: if fill { glium::draw_parameters::PolygonMode::Fill } else { glium::draw_parameters::PolygonMode::Line },
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };
        target.draw(&self.vertex_buffer, &self.index_buffer, &renderer.program, &uniform!{ win_size: (w as f32, h as f32) }, &params)
    }
}
