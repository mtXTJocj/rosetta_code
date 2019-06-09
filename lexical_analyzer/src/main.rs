use lexical_analyzer::error::Result;
use lexical_analyzer::token::TokenKind;
use lexical_analyzer::LexicalAnalyzer;

use std::env;
use std::fs::File;
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Write};

fn analyze(src: String, out: &mut Write) -> Result<()> {
    let mut lex = LexicalAnalyzer::new(src.chars());
    loop {
        let token = lex.next_token()?;

        out.write_fmt(format_args!("{}\n", token)).unwrap();

        if *token.kind() == TokenKind::EndOfInput {
            break;
        }
    }

    Ok(())
}

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
    reader
        .read_to_string(&mut str_in)
        .expect("cannot read source code");

    analyze(str_in, &mut writer).expect("lexcal analyzer failed.");
}
