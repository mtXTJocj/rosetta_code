use lexical_analyzer::error::Result;
use lexical_analyzer::token::TokenKind;
use lexical_analyzer::LexicalAnalyzer;

use std::env;
use std::fs::File;
use std::io::{stdin, stdout, BufReader, BufWriter, Read, Write};

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
    let args: Vec<String> = env::args().collect();
    let mut str_in = String::new();

    match args.len() {
        1 => {
            stdin()
                .lock()
                .read_to_string(&mut str_in)
                .expect("read failed.");
            let out = stdout();
            let mut out_lock = out.lock();
            analyze(str_in, &mut out_lock).expect("lexcal analyzer failed.");
        }
        2 => {
            let f = File::open(&args[1]).expect("open error");
            let mut f = BufReader::new(f);
            f.read_to_string(&mut str_in).expect("file read error");

            let out = stdout();
            let mut out_lock = out.lock();
            analyze(str_in, &mut out_lock).expect("lexcal analyzer failed.");
        }
        3 => {
            let f = File::open(&args[1]).expect("open error");
            let mut f = BufReader::new(f);
            f.read_to_string(&mut str_in).expect("file read error");

            let f = File::create(&args[2]).expect("open error");
            let mut f = BufWriter::new(f);
            analyze(str_in, &mut f).expect("lexcal analyzer failed.");
        }
        _ => return,
    };
}
