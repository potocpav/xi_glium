
use std::sync::mpsc;
use std::thread;
use std::process::{Stdio,Command,ChildStdin};
use std::io::BufReader;
use std::io::prelude::*;

use serde_json::{self,Value};
use serde_json::builder::*;

macro_rules! println_err (
    ($($arg:tt)*) => { {
        writeln!(&mut ::std::io::stderr(), $($arg)*).expect("failed printing to stderr");
    } }
);

pub struct Core {
    stdin: ChildStdin,
    pub rx: mpsc::Receiver<Value>,
    rpc_rx: mpsc::Receiver<Value>, // ! A simple piping works only for synchronous calls.
    rpc_index: u64,
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
        let (rpc_tx, rpc_rx) = mpsc::channel();
        let mut stdout = process.stdout.unwrap();
        thread::spawn(move || {
            let mut size_buf = [0u8; 8];
            while stdout.read_exact(&mut size_buf).is_ok() {
                let size = decode_u64(&size_buf) as usize;
                // println!("Size: {}", size);
                let mut buf = vec![0; size as usize];
                if stdout.read_exact(&mut buf).is_ok() {
                    if let Ok(data) = serde_json::from_slice::<Value>(&buf) {
                        {
                            let arr = data.as_array().unwrap();
                            if arr[0].as_string().unwrap() == "rpc_response" {
                                // catch an rpc response to an internal pipe
                                rpc_tx.send(arr[1].clone());
                                continue;
                            }
                        }
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

        Core { stdin: stdin, rx: rx, rpc_rx: rpc_rx, rpc_index: 0 }
    }

    pub fn save(&mut self, filename: &str) {
        self.write(ArrayBuilder::new().push("save").push(filename).unwrap());
    }

    pub fn open(&mut self, filename: &str) {
        self.write(ArrayBuilder::new().push("open").push(filename).unwrap());
    }

    fn send_char(&mut self, c: char) {
        self.write(ArrayBuilder::new().push("key").push_object(|builder| builder.insert("chars", c).insert("flags", 0)).unwrap());
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
        self.write(ArrayBuilder::new().push("scroll").push_array(|builder| builder.push(start).push(end)).unwrap());
    }

    pub fn click(&mut self, line: u64, column: u64) {
        self.write(ArrayBuilder::new().push("click").push_array(|builder| builder
            .push(line).push(column).push(0).push(1)
        ).unwrap());
    }

    pub fn test(&mut self) {
        self.render_lines(0, 10);
        // println!("test");
        // self.send_char('\u{F703}\", \"flags\": 0}]".as_bytes());
        // self.write(ArrayBuilder::new().push("scroll").push(filename).unwrap());
        // self.write(r#"["scroll", [5, 20]]"#.as_bytes()).unwrap();
    }

    pub fn render_lines(&mut self, start: u64, end: u64) {
        self.rpc_index += 1;
        println!("render_lines");
        let value = ArrayBuilder::new()
            .push("rpc")
            .push_object(|builder| builder
                .insert("index", self.rpc_index)
                .insert_array("request", |builder| builder
                    .push("render_lines")
                    .push_object(|builder| builder
                        .insert("first_line", start)
                        .insert("last_line", end)
                    )
                )
            ).unwrap();
        self.write(value);
    }

    pub fn render_lines_sync(&mut self, start: u64, end: u64) -> Value {
        self.render_lines(start, end);
        let value = self.rpc_rx.recv().unwrap();
        let object = value.as_object().unwrap();
        assert_eq!(self.rpc_index, object.get("index").unwrap().as_u64().unwrap());
        object.get("result").unwrap().clone()
    }

    fn write(&mut self, message: Value) {
        let str_msg = serde_json::ser::to_string(&message).unwrap();
        self.stdin.write(&encode_u64(str_msg.len() as u64)).unwrap();
        self.stdin.write(&str_msg.as_bytes()).unwrap();
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
