
use std::path::Path;
use std::fs::File;

use glium;
use glium_text;
use glium::Surface;
use glium::index::PrimitiveType;

use text::Line;
use controller::State;

pub struct Target<'a> {
    target: glium::Frame,
    renderer: &'a Renderer,
}

impl<'a> Target<'a> {

    pub fn draw_line(&mut self, line: &Line, (px, py): (f32, f32), line_nr: u64)
            -> Result<(), glium::DrawError> {
        let (renderer, target) = (&self.renderer, &mut self.target);
        let size = self.renderer.font_texture.em_pixels();
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

        let text = glium_text::TextDisplay::new(&renderer.text_system, &renderer.font_texture, &line.text);

        if let Some(mut pos) = line.cursor {
            let ch_pos_x = text.get_char_pos_x();
            assert!(ch_pos_x.len() > pos as usize);
            let offset_local = ch_pos_x[pos as usize];
            let offset_screen = offset_local * size as f32;

            renderer.line_bg.draw(target, &renderer.program, (px, py)).unwrap();
            renderer.cursor.draw(target, &renderer.program, (offset_screen + px, py)).unwrap();
        }

        glium_text::draw(&text, &renderer.text_system, target, text_tf(px, py), (0., 0., 0., 1.));

        Ok(())
    }

    pub fn finish(self) {
        self.target.finish().unwrap();
    }
}

pub struct Renderer {
    program: glium::Program,
    text_system: glium_text::TextSystem,
    font_texture: glium_text::FontTexture,

    cursor: Primitive,
    line_bg: Primitive,
}

impl Renderer {
    pub fn new(display: &glium::backend::glutin_backend::GlutinFacade) -> Renderer {
        let font_size = 15;

        let text_system = glium_text::TextSystem::new(display);
        let font_texture = glium_text::FontTexture::new(display, File::open(&Path::new("Hack-Regular.ttf")).unwrap(), font_size).unwrap();

        let program = program!(display,
            140 => {
                vertex: "
                    #version 140
                    in vec2 position;
                    in vec4 color;
                    out vec4 v_color;
                    uniform vec2 win_size;
                    uniform vec2 offset;
                    void main() {
                        v_color = color;
                        gl_Position = vec4((position + offset) / win_size * 2. - 1., 0.0, 1.0);
                    }
                ",
                fragment: "
                    #version 140
                    in vec4 v_color;
                    out vec4 color;
                    void main() {
                        color = v_color;
                    }
                "
            },
            110 => {
                vertex: "
                    #version 110

                    attribute vec2 position;
                    attribute vec4 color;
                    varying vec4 v_color;

                    uniform vec2 win_size;
                    uniform vec2 offset;

                    void main() {
                        v_color = color;
                        gl_Position = vec4((position + offset) / win_size * 2. - 1., 0.0, 1.0);
                    }
                ",
                fragment: "
                    #version 110

                    varying vec4 v_color;

                    void main() {
                        gl_FragColor = v_color;
                    }
                "
        }).unwrap();

        let cursor = Primitive::new_line(display, (0.,-10.), (0.,10.), [0.,0.,0.,1.]);
        let line_bg = Primitive::new_rect(display, (0., -10.), (2000., 10.), [1.,1.,0.7,1.]);

        Renderer {
            program: program,
            text_system: text_system,
            font_texture: font_texture,
            cursor: cursor,
            line_bg: line_bg,
        }
    }

    pub fn draw(&self, display: &glium::backend::glutin_backend::GlutinFacade) -> Target {
        let mut target = display.draw();
        target.clear_color(1.0, 1.0, 1.0, 0.0);
        Target { target: target, renderer: &self }
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
    fill: bool,
}

impl Primitive {
    pub fn new(display: &glium::Display, verts: &[Vertex], primitive_type: glium::index::PrimitiveType, fill: bool) -> Self {
        Primitive {
            vertex_buffer: glium::VertexBuffer::new(display, verts).unwrap(),
            index_buffer: glium::index::NoIndices(primitive_type),
            fill: fill,
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
            fill: true,
        }
    }

    pub fn new_line(display: &glium::Display, p1: (f32,f32), p2: (f32,f32), color: [f32; 4]) -> Self {
        let verts = vec![
            Vertex { position: [p1.0, p1.1], color: color },
            Vertex { position: [p2.0, p2.1], color: color },
        ];
        Primitive {
            vertex_buffer: glium::VertexBuffer::new(display, &verts).unwrap(),
            index_buffer:  glium::index::NoIndices(PrimitiveType::LinesList),
            fill: false,
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

    pub fn draw(&self, target: &mut glium::Frame, program: &glium::Program, offset: (f32, f32)) -> Result<(), glium::DrawError> {
        let (w, h) = target.get_dimensions();
        let params = glium::DrawParameters {
            polygon_mode: if self.fill { glium::draw_parameters::PolygonMode::Fill } else { glium::draw_parameters::PolygonMode::Line },
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };
        target.draw(&self.vertex_buffer, &self.index_buffer, program, &uniform!{ win_size: (w as f32, h as f32), offset: offset }, &params)
    }
}
