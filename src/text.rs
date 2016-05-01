use std::collections::BTreeMap;
use serde_json::Value;

// use renderer::Renderer;

const LINE_HEIGHT: f32 = 20.;

#[derive(Clone)]
pub struct Line {
    pub text: String,
    pub cursor: Option<u64>,
}

impl Line {
    pub fn placeholder() -> Line { Line { text: ">>> NOT IN CACHE <<<".into(), cursor: None } }
}

pub struct Text {
    cache: BTreeMap<u64, Line>,
    pub top: f64,
    pub height: f64,
    n_lines: u64,
}

impl Text {
    pub fn new() -> Text {
        Text { cache: BTreeMap::new(), top: 0., height: 0., n_lines: 0}
    }

    pub fn refresh(&mut self, n_lines: u64) {
        self.cache.clear();
        self.n_lines = n_lines;
    }

    // pub fn render(&self, renderer: &mut Renderer) {
    //     unimplemented!()
    //     // TODO: move the rendering logic into this module
    // }

    pub fn add_lines(&mut self, value: &Value, first: u64) {
        for (i, line) in value.as_array().unwrap().into_iter().enumerate() {
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
            self.cache.insert(i as u64+first, Line { text: text, cursor: cursor });
        }
    }

    pub fn scroll_to(&mut self, line: u64, _column: u64) {
        let min = |a,b| if a > b { b } else { a };
        let max = |a,b| if a < b { b } else { a };
        self.top = max(0., min(self.top, line as f64 - 2.)); // scroll up
        self.top = min(self.n_lines as f64, max(self.top, line as f64 - self.height + 1. + 2.)); // scroll dn
    }

    pub fn get_lines(&self) -> Vec<(f32, Line)> {
        (self.top as u64 .. (self.top + self.height).ceil() as u64)
            .filter_map(|i| self.get_line(i).map(|l|
                ((self.height - i as f64 + self.top - 0.5) as f32 * LINE_HEIGHT, l)
            )).collect()
    }

    fn get_line(&self, n: u64) -> Option<Line> {
        if n >= self.n_lines {
            None
        } else {
            Some(self.cache.get(&n).cloned().unwrap_or(Line::placeholder()))
        }
    }

    pub fn set_size(&mut self, _w: u32, h: u32) {
        self.height = h as f64 / LINE_HEIGHT as f64;
    }
}
