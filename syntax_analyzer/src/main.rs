use lexical_analyzer::token::*;
use syntax_analyzer::SyntaxAnalyzer;

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

    let mut str_in = String::new();
    reader.read_to_string(&mut str_in).expect("read error");

    let tokens: Vec<Token> = str_in
        .lines()
        .map(|line| Token::from_line(line).unwrap())
        .collect();

    let ast = SyntaxAnalyzer::parse(tokens.into_iter()).expect("parser failed");
    writer
        .write_fmt(format_args!("{}", ast))
        .expect("write failed");
}
