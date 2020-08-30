mod parser;
mod combinator;

use std;
mod stream;
mod tokenizer;

fn main() {
    let mut args = std::env::args();
    args.next();
    for argument in args {
        let i_text = std::fs::read(&argument).unwrap();
        let mut is = stream::IStream::new(&i_text, &argument);
        let tk = tokenizer::Tokenizer::new(&mut is);
        let parser = parser::Parser::new(tk.get_tokens());
        println!("{:?}", parser.get_statements());
    }
}