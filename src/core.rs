
use std::sync::mpsc;
use std::thread;
use std::process::{Stdio,Command,ChildStdin};
use std::io::BufReader;
use std::io::prelude::*;

use serde_json::{self,Value};

macro_rules! println_err (
    ($($arg:tt)*) => { {
        writeln!(&mut ::std::io::stderr(), $($arg)*).expect("failed printing to stderr");
    } }
);

pub struct Core {
    stdin: ChildStdin,
    pub rx: mpsc::Receiver<Value>,
}

impl Core {
    pub fn new(executable: &str) -> Core {
        // spawn the core process
        let process = Command::new(executable)
                                .arg("test-file")
                                .stdout(Stdio::piped())
                                .stdin(Stdio::piped())
                                .stderr(Stdio::piped())
                                .env("RUST_BACKTRACE", "1")
                                .spawn()
                                .unwrap_or_else(|e| { panic!("failed to execute core: {}", e) });


        let (tx, rx) = mpsc::channel();
        let mut stdout = process.stdout.unwrap();
        thread::spawn(move || {
            let mut size_buf = [0u8; 8];
            while stdout.read_exact(&mut size_buf).is_ok() {
                let size = decode_u64(&size_buf) as usize;
                // println!("Size: {}", size);
                let mut buf = vec![0; size as usize];
                if stdout.read_exact(&mut buf).is_ok() {
                    if let Ok(data) = serde_json::from_slice::<Value>(&buf) {
                        // println!("from core: {:?}", data);
                        tx.send(data).unwrap();
                    }
                }
            }
        });

        let stderr = process.stderr.unwrap();
        thread::spawn(move || {
            let buf_reader = BufReader::new(stderr);
            for line in buf_reader.lines() {
                if let Ok(line) = line {
                    println_err!("[core] {}", line);
                }
            }
        });

        let stdin = process.stdin.unwrap();

        Core { stdin: stdin, rx: rx }
    }

    pub fn save(&mut self, filename: &str) {
        self.write(format!("[\"save\", \"{}\"]", filename).as_bytes()).unwrap();
    }


    // TODO: construct the JSON safely!!
    pub fn open(&mut self, filename: &str) {
        self.write(format!("[\"open\", \"{}\"]", filename).as_bytes()).unwrap();
    }

    fn send_char(&mut self, c: char) {
        self.write(format!(r#"["key", {{ "chars": "{}", "flags": 0}}]"#, c).as_bytes()).unwrap();
    }

    pub fn left(&mut self) { self.send_char('\u{F702}'); }

    pub fn right(&mut self) { self.send_char('\u{F703}'); }

    pub fn up(&mut self) { self.send_char('\u{F700}'); }

    pub fn down(&mut self) { self.send_char('\u{F701}'); }

    pub fn del(&mut self) { self.send_char('\x7f'); }

    pub fn page_up(&mut self) { self.send_char('\u{F72C}'); }

    pub fn page_down(&mut self) { self.send_char('\u{F72D}'); }

    pub fn f1(&mut self) { self.send_char('\u{F704}'); }

    pub fn f2(&mut self) { self.send_char('\u{F705}'); }

    pub fn char(&mut self, ch: char) { self.send_char(ch); }

    pub fn scroll(&mut self, start: u64, end: u64) {
        // println!("test");
        // self.send_char('\u{F703}\", \"flags\": 0}]".as_bytes());
        self.write(format!(r#"["scroll", [{}, {}]]"#, start, end).as_bytes()).unwrap();
    }

    pub fn test(&mut self) {
        // println!("test");
        // self.send_char('\u{F703}\", \"flags\": 0}]".as_bytes());
        self.write(r#"["scroll", [5, 20]]"#.as_bytes()).unwrap();
    }
    //
    // pub fn render_lines(&mut self) {
    //     println!("render_lines");
    //     self.write(r#"["rpc", {"index": "1", "request": ["render_lines", { "first_line": 0, "last_line": 10}]}]"#.as_bytes()).unwrap();
    // }

    fn write(&mut self, message: &[u8]) -> ::std::io::Result<usize> {
        Ok(
            try!(self.stdin.write(&encode_u64(message.len() as u64))) +
            try!(self.stdin.write(message))
        )
    }
}

fn encode_u64(n: u64) -> [u8; 8] {
    let mut sizebuf = [0; 8];
    for i in 0..8 {
        sizebuf[i] = (((n as u64) >> (i * 8)) & 0xff) as u8;
    }
    sizebuf
}

fn decode_u64(array: &[u8]) -> u64 {
    array.iter().enumerate().fold(0, |s, (i, &b)| s + ((b as u64) << (i * 8)))
}
