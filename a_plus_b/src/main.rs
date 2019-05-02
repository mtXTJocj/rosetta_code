use std::io::Write;

fn main() {
    let mut line = String::new();

    // プロンプト出力
    print!("input A and B: ");
    std::io::stdout().flush().ok();

    // 入力受け付け
    std::io::stdin()
        .read_line(&mut line)
        .expect("reading stdin");

    let result: i32 = line
        .trim()
        .split_whitespace()
        .map(|x| x.parse::<i32>().expect("not an integer"))
        .sum();

    println!("A + B = {}", result);
}
