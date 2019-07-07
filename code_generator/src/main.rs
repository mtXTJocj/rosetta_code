use code_generator::CodeGenerator;
use syntax_analyzer::ast_node::*;

use std::env;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

fn main() {
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

    let mut ast_str = String::new();
    reader.read_to_string(&mut ast_str).expect("read error");
    let ast = ASTReader::read_ast(ast_str.lines());
    let code = CodeGenerator::generate(&ast).unwrap();

    writer.write(code.as_bytes()).expect("write error");
}
