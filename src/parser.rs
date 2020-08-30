use crate::tokenizer::{
    Token,
    Literal,
    Keyword,
    Identifier,
    Operator,
    Separator,
    Type,
    get_type,
    get_v_description,
    get_literal,
};

#[derive(Debug)]
pub enum Statement {
    NewConst(Identifier, Literal, Option<(Type, usize)>),
    NewLet(Identifier, Option<Literal>, Option<(Type, usize)>)
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
        for _ in 0..10 {
            match self.find_statement() {
                Some(s) => self.statements.push(s),
                None => {}
            }
        }
    }

    pub fn find_statement(&mut self) -> Option<Statement> {
        match self.next() {
            Token::Keyword(kw) => match kw {
                Keyword::Const | Keyword::Let => {
                    let kw_copy = kw.clone();
                    let var = match self.next() {
                        Token::Identifier(Identifier::Variable(n)) => n.clone(),
                        _ => panic!("const variable is invalid")
                    };
                    let var_description = self.take_to_tokens(&[
                        Token::Operator(Operator::Assign),
                        Token::Newline,
                    ]);
                    let description: Option<(Type, usize)> = if var_description.len() != 0 {
                        Some(get_v_description(&var_description))
                    } else {
                        None
                    };
                    let mut literal_tokens = self.take_to_tokens(&[
                        Token::Newline,
                    ]);
                    match kw_copy {
                        Keyword::Const => {
                            assert_eq!(*literal_tokens.first().unwrap(), Token::Operator(Operator::Assign));
                            literal_tokens.remove(0);
                            Some(Statement::NewConst(Identifier::Variable(var), get_literal(&literal_tokens), description))
                        },
                        Keyword::Let => {
                            // let literal = if self.peek().unwrap() == Token::Operator(Operator::Assign) {
                            //     self.next();
                            //     Some(get_literal(&self.take_to_tokens(&[Token::Newline])))
                            // } else {
                            //     None
                            // };
                            // Some(Statement::NewLet(Identifier::Variable(var), literal, description))
                            let literal = match &self.peek().unwrap() {
                                Token::Operator(Operator::Assign) => {
                                    self.next();
                                    Some(get_literal(&self.take_to_tokens(&[Token::Newline])))
                                },
                                _ => None,
                            };
                            Some(Statement::NewLet(Identifier::Variable(var), literal, description))
                        },
                        _ => panic!("impossible")
                    }
                }
                _ => None
            }
            _ => None
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