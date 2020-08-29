use crate::tokenizer::{
    Token,
    Literal,
    Keyword,
    Identifier,
    Operator,
    Separator,
};

#[derive(Debug)]
pub enum Statement {
    NewConst(Identifier, Literal)
}

enum Expression<'a> {
    Binary { left: &'a Expression<'a>, right: &'a Expression<'a>, operator: Operator },
    Unary { operand: &'a Expression<'a>, operator: Operator },
}

pub struct Parser<'a> {
    statements: Vec<Statement>,
    tokens: &'a Vec<Token>,
    idx: usize,
}

impl Parser<'_> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        let mut parser = Parser {
            tokens,
            statements: Vec::new(),
            idx: 0,
        };
        parser.find_statements();
        parser
    }

    pub fn next(&mut self) -> &Token {
        let ntoken: &Token = &self.tokens[self.idx];
        self.idx += 1;
        return ntoken;
    }

    pub fn peek(&self) -> Option<&Token> {
        if self.idx > self.tokens.len() - 1 {
            None
        } else {
            Some(&self.tokens[self.idx])
        }
    }

    pub fn find_statements(&mut self) {
        for i in 0..10 {
            self.find_statement()
        }
    }

    pub fn find_statement(&mut self) {
        match self.next() {
            Token::Keyword(kw) => match kw {
                Keyword::Const => {
                    let var = self.next();
                    match var {
                        Token::Identifier(Identifier::Variable(_)) => {}
                        _ => panic!("const variable is invalid")
                    }
                    let var_type = self.take_to_tokens(&[
                        Token::Operator(Operator::Assign),
                        Token::Newline,
                    ]);
                    if var_type.len() != 0 {
                        // type was found
                    }
                    assert_eq!(*self.next(), Token::Operator(Operator::Assign));
                    let literal = self.take_to_tokens(&[
                        Token::Newline,
                    ]);
                    // let expr = self.parse_expression(&assignment);
                }
                Keyword::Let => {}
                _ => {}
            }
            _ => {}
        }
    }

    // pub fn parse_expression(&self, expression: &Vec<Token>) -> Expression {}

    pub fn get_statements(&self) -> &Vec<Statement> {
        &self.statements
    }

    pub fn take_to_tokens(&mut self, tks: &[Token]) -> Vec<Token> {
        let taken_tokens = self.tokens[self.idx..].iter().take_while(|curr_token| {
            for tk in tks {
                if tk == *curr_token {
                    return false;
                }
            }
            return true;
        });
        let res: Vec<Token> = taken_tokens.cloned().collect();
        self.idx += res.len();
        res
    }
}