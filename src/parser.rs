use crate::tokenizer::{
    Token,
    Literal,
    Keyword,
    Identifier,
    Operator,
    Separator,
    Type,
    get_v_description,
    get_literal,
};

type Scope = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum Statement {
    NewConst { name: Identifier, literal: Literal, v_type: Option<(Type, usize)>},
    NewLet { name: Identifier, literal: Option<Literal>, v_type: Option<(Type, usize)>},
    Fn { f: Function, scope: Scope },
    Scope { scope: Scope }
}

enum Expression<'a> {
    Binary { left: &'a Expression<'a>, right: &'a Expression<'a>, operator: Operator },
    Unary { operand: &'a Expression<'a>, operator: Operator },
}


#[derive(Debug, Clone)]
pub struct Function {
    name: Vec<u8>,
    external: bool,
}


pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    idx: usize,
    current_fn: Option<Function>,
    scope_stack: Vec<Scope>,
}

impl Parser<'_> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        let mut parser = Parser {
            tokens,
            idx: 0,
            current_fn: None,
            scope_stack: vec![vec![]],
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
        while self.tokens[self.idx] != Token::Eof {
            match self.get_statement() {
                Some(s) => self.add_statement(s),
                None => {},
            }
        }
    }

    pub fn get_statement(&mut self) -> Option<Statement> {
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
                            Some(Statement::NewConst {
                                name: Identifier::Variable(var),
                                literal: get_literal(&literal_tokens),
                                v_type: description,
                            })
                        },
                        Keyword::Let => {
                            let literal = match &self.peek().unwrap() {
                                Token::Operator(Operator::Assign) => {
                                    self.next();
                                    Some(get_literal(&self.take_to_tokens(&[Token::Newline])))
                                },
                                _ => None,
                            };
                            Some(Statement::NewLet {
                                literal,
                                name: Identifier::Variable(var),
                                v_type: description,
                            })
                        },
                        _ => panic!("impossible")
                    }
                }
                Keyword::External => {
                    // for now, only functions can be global
                    assert_eq!(*self.next(), Token::Keyword(Keyword::Function));
                    let name_token = self.next().clone();
                    self.create_function(&name_token, true);
                    None
                },
                Keyword::Function => {
                    let name_token = self.next().clone();
                    self.create_function(&name_token, false);
                    None
                },
                _ => None
            }
            Token::Separator(sep) => match sep {
                Separator::CloseBrace => {
                    // because the scope after a global scope is always a function's scope
                    // we can assume if there's a current function and a 2nd scope the 2nd scope
                    // belongs to the function
                    match &self.current_fn {
                        Some(f) => {
                            if self.scope_stack.len() == 2 {
                                let function_scope = self.scope_stack.pop().unwrap();
                                self.add_statement(Statement::Fn {
                                    f: f.clone(),
                                    scope: function_scope,
                                });
                                self.current_fn = None;
                            }
                        },
                        None => {}
                    }
                    None
                },
                Separator::OpenBrace => {
                    self.scope_stack.push(vec![]);
                    None
                }
                _ => None,
            }
            Token::Identifier(id) => match id {
                Identifier::Register(r) => {
                    None
                },
                _ => None,
            },
            _ => None
        }
    }

    pub fn create_function(&mut self, name: &Token, external: bool) {
        match &self.current_fn {
            Some(_old_f) => panic!("no nested functions"),
            None => {
                self.current_fn = Some(Function {
                    external,
                    name: match name {
                        Token::Identifier(Identifier::Variable(n)) => n.clone(),
                        _ => panic!("token is bad"),
                    },
                });
            },
        };
    }

    pub fn add_statement(&mut self, s: Statement) {
        let scope = &mut self.scope_stack.pop().unwrap();
        scope.push(s);
        self.scope_stack.push(scope.to_vec());
    }

    pub fn get_statements(&self) -> &Vec<Statement> {
        &self.scope_stack.first().unwrap()
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