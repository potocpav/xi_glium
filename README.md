# xi glium

**Xi glium** is a `glium` and `glium_text`-based interface on top of the
[**xi editor**](https://github.com/google/xi-editor) project back-end.

Screenshot:
![xi glium](/screenshot.png?raw=true)

## Features

* Write and backspace text
* Load (`ctrl-o`) and save (`ctrl-s`) to a file specified as an argument
* Navigate using arrows

You must specify a path to the `xicore` executable (build by cargo inside
the `rust` subdirectory of xi-editor). Works with the xi-editor commit f2a567b.

## Example usage

`xicore=../rust/target/debug/xicore cargo run README.md`
