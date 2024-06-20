use std::{env, fs, io::Read};

use ccr_core::lexer::Scanner;

fn main() {
    let name = env::args().nth(1).expect("__CFILE");

    let mut buf = Vec::new();
    let mut file = fs::File::open(name).expect("Ohno cannot open file");
    file.read_to_end(&mut buf).expect("Fuck");

    let buf = std::str::from_utf8(&buf).expect("ff");

    let mut scanner = Scanner::new(buf);
    let stream = scanner.tokenize();

    for s in stream.iter() {
        println!("{s:?}");
    }
}
