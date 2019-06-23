use ast_interpreter::ASTInterpreter;
use syntax_analyzer::ast_node::*;

use std::env;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

fn main() {
    let mut str_in = String::new();

    let mut reader: Box<dyn BufRead> = match env::args().nth(1) {
        None => Box::new(BufReader::new(stdin())),
        Some(filename) => Box::new(BufReader::new(
            File::open(filename).expect("cannot open file"),
        )),
    };

    let mut writer: Box<dyn Write> = match env::args().nth(2) {
        None => Box::new(BufWriter::new(stdout())),
        Some(filename) => Box::new(BufWriter::new(
            File::create(filename).expect("cannot create file"),
        )),
    };

    reader.read_to_string(&mut str_in).expect("read failed");
    let ast = ASTReader::read_ast(str_in.lines());
    match ASTInterpreter::interpret(&ast, &mut writer) {
        Ok(_) => {}
        Err(e) => println!("{:?}", e),
    }
}
