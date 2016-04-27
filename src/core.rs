
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
    _stdout_handle: thread::JoinHandle<()>,
    _stderr_handle: thread::JoinHandle<()>,
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
        let _stdout_handle = thread::spawn(move || {
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
        let _stderr_handle = thread::spawn(move || {
            let buf_reader = BufReader::new(stderr);
            for line in buf_reader.lines() {
                if let Ok(line) = line {
                    println_err!("[core] {}", line);
                }
            }
        });


        let stdin = process.stdin.unwrap();
        // stdin_handle.write("[\"open\", \"test-file\"]\n".as_bytes());

        Core { _stdout_handle: _stdout_handle, _stderr_handle: _stderr_handle, stdin: stdin, rx: rx }
        // unimplemented!()
    }

    // // TODO: delete. just an infinite wait
    // pub fn join(self) {
    //     self._stdout_handle.join().unwrap();
    // }
    //

    pub fn save(&mut self, filename: &str) {
        self.write(format!("[\"save\", \"{}\"]", filename).as_bytes()).unwrap();
    }

    // TODO: construct safely!!
    pub fn open(&mut self, filename: &str) {
        self.write(format!("[\"open\", \"{}\"]", filename).as_bytes()).unwrap();
    }

    pub fn left(&mut self) {
        self.write("[\"key\", { \"chars\": \"\u{F702}\", \"flags\": 0}]".as_bytes()).unwrap();
    }

    pub fn right(&mut self) {
        self.write("[\"key\", { \"chars\": \"\u{F703}\", \"flags\": 0}]".as_bytes()).unwrap();
    }

    pub fn up(&mut self) {
        self.write("[\"key\", { \"chars\": \"\u{F700}\", \"flags\": 0}]".as_bytes()).unwrap();
    }

    pub fn down(&mut self) {
        self.write("[\"key\", { \"chars\": \"\u{F701}\", \"flags\": 0}]".as_bytes()).unwrap();
    }

    pub fn del(&mut self) {
        self.write("[\"key\", { \"chars\": \"\x7f\", \"flags\": 0}]".as_bytes()).unwrap();
    }

    pub fn char(&mut self, ch: char) {
        self.write(format!("[\"key\", {{ \"chars\": \"{}\", \"flags\": 0}}]", ch).as_bytes()).unwrap();
    }

    // pub fn test(&mut self) {
    //     println!("test");
    //     self.write("[\"key\", { \"chars\": \"\u{F703}\", \"flags\": 0}]".as_bytes());
    //     self.write(r#"["scroll", ["0", "10"]]"#.as_bytes());
    // }
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
