mod compiler;
mod parser;
mod combinator;

use std;
use std::str::from_utf8;

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
        let mut program = compiler::Program::new();
        program.compile(parser.scope_stack);
        println!("{}", from_utf8(&*program.header).unwrap());
        println!("{}", from_utf8(&*program.text).unwrap());
        println!("{}", from_utf8(&*program.ro_data).unwrap());
        println!("{}", from_utf8(&*program.bss).unwrap());
    }
}