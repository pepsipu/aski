use std::str;

use crate::stream::{
    IStream,
};
use crate::combinator;


pub struct Tokenizer<'a> {
    istream: &'a mut IStream<'a>,
    tokens: Vec<Token>,
}

impl Tokenizer<'_> {
    pub fn new<'a>(istream: &'a mut IStream<'a>) -> Tokenizer<'a> {
        let tokens = Vec::<Token>::new();
        let mut tokenizer = Tokenizer {
            istream,
            tokens,
        };
        tokenizer.find_tokens();
        tokenizer
    }

    pub fn get_tokens(&self) -> &Vec<Token> {
        &self.tokens
    }

    fn find_tokens(&mut self) {
        loop {
            let token: Token = self.take_token();
            match token {
                Token::Eof => break,
                _ => {}
            }
            self.tokens.push(token);
        }
        self.tokens.push(Token::Eof);
    }

    fn take_token(&mut self) -> Token {
        // clear ws
        self.istream.take_while(&combinator::is_ws);
        if self.istream.empty {
            return Token::Eof;
        }
        let peek: u8 = self.istream.peek(0).unwrap();
        if peek == b'\n' {
            self.istream.take_to_c(b'\n');
            return Token::Newline;
        }
        // if comment, we should read until the end of the line
        if peek == b';' {
            self.istream.take_to_c(b'\n');
            return self.take_token();
        }
        if combinator::is_quote(peek, None) {
            // take open quote
            self.istream.next();
            let s: Vec<u8> = self.istream.take_while(&|c: u8, tk: Option<&mut IStream>| -> bool {
                // we should keep taking if !quote or it can be a quote but the prev char is \
                !combinator::is_quote(c, None) || tk.unwrap().peek(-1).unwrap() == b'\\'
            });
            // take closing quote
            self.istream.next();
            Token::Literal(Literal::String(s))
        } else if combinator::is_int(peek, None) {
            let num = self.istream.take_while(&combinator::is_int);
            let num_str: &str = str::from_utf8(&num).unwrap();
            Token::Literal(Literal::Int(num_str.parse::<usize>().unwrap()))
        } else if combinator::is_kw_or_var(peek, None) {
            let kw_or_var = self.istream.take_while(&combinator::is_kw_or_var);
            match get_kw(&kw_or_var) {
                Some(kw) => Token::Keyword(kw),
                None => Token::Identifier(Identifier::Variable(kw_or_var))
            }
        } else if combinator::is_op(peek, None) {
            let op = self.istream.take_while(combinator::is_op);
            match get_op(&op) {
                Some(op) => Token::Operator(op),
                None => {
                    self.istream.err();
                    Token::Eof
                }
            }
        } else if peek == b'$' {
            self.istream.next();
            if self.istream.peek(0).unwrap() == b'*' {
                self.istream.next();
                let reg = self.istream.take_while(combinator::is_register_name);
                Token::Identifier(Identifier::DerefRegister(reg))
            } else {
                let reg = self.istream.take_while(combinator::is_register_name);
                Token::Identifier(Identifier::Register(reg))
            }
        } else {
            let sep = get_sep(peek);
            match sep {
                Some(separator) => {
                    self.istream.next();
                    Token::Separator(separator)
                }
                None => {
                    self.istream.err();
                    Token::Eof
                }
            }
        }
    }
}


pub fn get_kw(s: &Vec<u8>) -> Option<Keyword> {
    // &Vec -> Vec -> [u8] -> &[u8] for cmp
    match &**s {
        b"const" => Some(Keyword::Const),
        b"let" => Some(Keyword::Let),
        b"fn" => Some(Keyword::Function),
        b"extern" => Some(Keyword::External),
        b"while" => Some(Keyword::While),
        b"call" => Some(Keyword::Call),
        b"sizeof" => Some(Keyword::SizeOf),
        _ => None
    }
}

pub fn get_op(s: &Vec<u8>) -> Option<Operator> {
    match &**s {
        b"=" => Some(Operator::Assign),
        b"+" => Some(Operator::Add),
        b"-" => Some(Operator::Subtract),
        b"/" => Some(Operator::Divide),
        b"*" => Some(Operator::Multiply),
        b"%" => Some(Operator::Modulus),
        b"--" => Some(Operator::Decrement),
        b"++" => Some(Operator::Increment),
        b"!" => Some(Operator::Not),
        b"!=" => Some(Operator::NotEqual),
        b"==" => Some(Operator::Equal),
        _ => None,
    }
}

pub fn get_sep(c: u8) -> Option<Separator> {
    match c {
        b'[' => Some(Separator::OpenBracket),
        b']' => Some(Separator::CloseBracket),
        b'{' => Some(Separator::OpenBrace),
        b'}' => Some(Separator::CloseBrace),
        b':' => Some(Separator::Colon),
        b'(' => Some(Separator::OpenParentheses),
        b')' => Some(Separator::CloseParentheses),
        b',' => Some(Separator::Comma),
        _ => None,
    }
}

pub fn get_type(t: &Vec<u8>) -> Option<Type> {
    match &**t {
        b"byte" => Some(Type::Uint8),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Eof,
    Newline,
    Literal(Literal),
    Identifier(Identifier),
    Keyword(Keyword),
    Operator(Operator),
    Separator(Separator),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(Vec<u8>),
    Int(usize),
    Array(Vec<Literal>, Type, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Identifier {
    Variable(Vec<u8>),
    Register(Vec<u8>),
    DerefRegister(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Assign,
    Add,
    Subtract,
    Divide,
    Multiply,
    Modulus,
    Decrement,
    Increment,
    Not,
    Equal,
    NotEqual,
}

#[derive(Debug, Clone, PartialEq)]
// for my own sanity, braces = { and brackets = [
pub enum Separator {
    OpenBracket,
    CloseBracket,
    OpenBrace,
    CloseBrace,
    Colon,
    OpenParentheses,
    CloseParentheses,
    Comma,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Keyword {
    Const,
    Let,
    Function,
    External,
    While,
    Call,
    SizeOf,
}
#[derive(Debug, Clone, PartialEq)]

pub enum Type {
    Uint8,
}

pub fn get_v_description(vt: &Vec<Token>) -> (Type, usize) {
    assert_eq!(vt.len(), 6);
    assert_eq!(vt[0], Token::Separator(Separator::Colon));
    assert_eq!(vt[1], Token::Separator(Separator::OpenBracket));
    let v_type = match &vt[2] {
        Token::Identifier(Identifier::Variable(t)) => get_type(t).unwrap(),
        _ => panic!("invalid type"),
    };
    assert_eq!(vt[3], Token::Separator(Separator::Comma));
    let v_size = match &vt[4] {
        Token::Literal(Literal::Int(i)) => i,
        _ => panic!("invalid size"),
    };
    assert_eq!(vt[5], Token::Separator(Separator::CloseBracket));
    (v_type, *v_size)
}

pub fn get_literal(vt: &Vec<Token>) -> Literal {
    assert_ne!(vt.len(), 0);
    match vt.first().unwrap() {
        Token::Literal(l) => l.clone(),
        // add support for constant arrays later on
        // _ => {
        //     assert_eq!(vt[0], Token::Separator(Separator::OpenBracket));
        //
        // }
        _ => panic!("literal not recognized"),
    }
}