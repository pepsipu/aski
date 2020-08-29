mod parser;
mod combinator;

use std;
mod stream;
mod tokenizer;

fn main() {
    let mut args = std::env::args();
    args.next();
    for argument in args {
        let itext = std::fs::read(&argument).unwrap();
        let mut is = stream::IStream::new(&itext, &argument);
        let tk = tokenizer::Tokenizer::new(&mut is);
        let mut parser = parser::Parser::new(tk.get_tokens());
        println!("{:?}", parser.get_statements());
    }
}
