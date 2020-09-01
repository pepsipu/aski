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
    RegisterAssign { register: Vec<u8>, expression: Expression },
    RegisterDerefAssign { register: Vec<u8>, expression: Expression },
    InlineAssembly { instructions: Vec<u8> },
    Call { f: Vec<u8> },
    Scoped { scoped: ScopeImpl },
}

#[derive(Debug, Clone)]
pub enum ScopeImplType {
    Global,
    Fn { name: Vec<u8>, external: bool },
    If { left: Expression, right: Expression, condition: Operator },
}

#[derive(Debug, Clone)]
pub struct ScopeImpl {
    pub(crate) scope_type: ScopeImplType,
    pub(crate) scope: Scope,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Binary { left: Box<Expression>, right: Box<Expression>, operator: Operator },
    Unary { operand: Box<Expression>, operator: Operator },
    Number { value: usize },
    Register { reg: Vec<u8> },
    Variable { var: Vec<u8> },
    SizeOf { var: Identifier },
}

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    idx: usize,
    pub scope_stack: Vec<ScopeImpl>,
}

const PRECEDENCE: [Operator; 3] = [Operator::Add, Operator::Subtract, Operator::Multiply];

impl Parser<'_> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        let mut parser = Parser {
            tokens,
            idx: 0,
            scope_stack: vec![ScopeImpl {
                scope_type: ScopeImplType::Global,
                scope: vec![]
            }],
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
                Keyword::Call => {
                    assert_eq!(Token::Separator(Separator::OpenParentheses), *self.next());
                    let f = self.next().clone();
                    assert_eq!(Token::Separator(Separator::CloseParentheses), *self.next());
                    Some(Statement::Call { f: if let Token::Identifier(Identifier::Variable(v)) = f {
                        v.clone()
                    } else {
                        panic!("bad call");
                    }})
                },
                Keyword::If => {
                    let left_tokens = self.take_to_tokens(&[
                        Token::Operator(Operator::Equal),
                        Token::Operator(Operator::NotEqual),
                    ]);
                    let left = self.parse_expression(left_tokens);
                    let condition = self.next().clone();
                    let right_tokens = self.take_to_tokens(&[
                        Token::Separator(Separator::OpenBrace),
                    ]);
                    let right = self.parse_expression(right_tokens);
                    let scope = ScopeImpl { scope_type: ScopeImplType::If {
                        left,
                        right,
                        condition: if let Token::Operator(o) = condition {
                            o.clone()
                        } else {
                            panic!("bad if statement");
                        }
                    }, scope: vec![] };
                    self.scope_stack.push(scope);
                    None
                },
                _ => None
            }
            Token::Separator(sep) => match sep {
                Separator::CloseBrace => {
                    let done_scope = self.scope_stack.pop().unwrap();
                    self.add_statement(Statement::Scoped { scoped: done_scope });
                    None
                },
                _ => None,
            }
            Token::Identifier(id) => match id {
                Identifier::Register(r) => {
                    let r_copy = r.to_vec();
                    assert_eq!(Token::Operator(Operator::Assign), *self.next());
                    let expression_tokens = self.take_to_tokens(&[Token::Newline]);
                    let expression = self.parse_expression(expression_tokens);
                    Some(Statement::RegisterAssign {
                        register: r_copy,
                        expression
                    })
                },
                Identifier::DerefRegister(r) => {
                    let r_copy = r.to_vec();
                    assert_eq!(Token::Operator(Operator::Assign), *self.next());
                    let expression_tokens = self.take_to_tokens(&[Token::Newline]);
                    let expression = self.parse_expression(expression_tokens);
                    Some(Statement::RegisterDerefAssign {
                        register: r_copy,
                        expression
                    })
                }
                _ => None,
            },
            Token::Inline(inline) => Some(Statement::InlineAssembly {
                instructions: inline.to_vec(),
            }),
            _ => None
        }
    }

    pub fn parse_expression(&self, tks: Vec<Token>) -> Expression {
        let op_pos = {
            let mut ret: Option<usize> = None;
            for check_op in PRECEDENCE.iter() {
                 if let Some(i) = tks.iter().position(|token| {
                    let op = if let Token::Operator(op) = token {
                        op
                    } else {
                        return false;
                    };
                    check_op == op
                 }) {
                     ret = Some(i);
                     break;
                 }
            }
            ret
        };
        match op_pos {
            Some(op) => {
                let left = &tks[..op];
                let right = &tks[op + 1..];
                Expression::Binary {
                    left: Box::new(self.parse_expression(left.to_vec())),
                    right: Box::new(self.parse_expression(right.to_vec())),
                    operator: if let Token::Operator(operator) = tks[op].clone() {
                        operator
                    } else {
                        panic!("impossible");
                    }
                }
            }
            None => {
                // binary expression not applicable
                // for now just check if it's a number
                match &tks[0] {
                    Token::Literal(Literal::Int(i)) => Expression::Number { value: *i },
                    Token::Identifier(Identifier::Register(r)) => Expression::Register {
                        reg: r.to_vec()
                    },
                    Token::Keyword(Keyword::SizeOf) => {
                        assert_eq!(Token::Separator(Separator::OpenParentheses), tks[1]);
                        assert_eq!(Token::Separator(Separator::CloseParentheses), tks[3]);
                        Expression::Variable {
                            var: if let Token::Identifier(Identifier::Variable(v)) = tks[2].clone() {
                                let mut vc = v.to_vec().to_ascii_uppercase();
                                vc.extend_from_slice(b"_LEN");
                                vc
                            } else {
                                panic!("bad token for sizeof")
                            },
                        }
                    },
                    Token::Identifier(Identifier::Variable(v)) => Expression::Variable { var: v.to_vec() },
                    _ => panic!("bad token")
                }
            }
        }
    }

    // pub fn r_parse_expression(tree: Expression) -> Expression {
    //     tree
    // }

    pub fn create_function(&mut self, name: &Token, external: bool) {
        let name = match name {
            Token::Identifier(Identifier::Variable(n)) => n.clone(),
            _ => panic!("token is bad"),
        };
        // check for nested functions
        if self.scope_stack.iter().any(|scope| match &scope.scope_type {
            ScopeImplType::Fn { name, external } => true,
            _ => false,
        }) {
            panic!("no nested functions");
        }
        self.scope_stack.push(ScopeImpl {
            scope: vec![],
            scope_type: ScopeImplType::Fn {
                name,
                external,
            }
        })

    }

    pub fn add_statement(&mut self, s: Statement) {
        let len = self.scope_stack.len();
        self.scope_stack[len - 1].scope.push(s);
    }

    pub fn get_statements(&self) -> &ScopeImpl {
        self.scope_stack.first().unwrap()
    }

    pub fn take_to_tokens(&mut self, tks: &[Token]) -> Vec<Token> {
        let taken = self.tokens[self.idx..].iter().take_while(|curr_token| {
            for tk in tks {
                if tk == *curr_token {
                    return false;
                }
            }
            return true;
        });
        let res: Vec<Token> = taken.cloned().collect();
        self.idx += res.len();
        res
    }

    pub fn take_given_to_tokens(given_tks: &Vec<Token>, tks: &[Token]) -> Vec<Token> {
        let taken = given_tks.iter().take_while(|curr_token| {
            for tk in tks {
                if tk == *curr_token {
                    return false;
                }
            }
            return true;
        });
        taken.cloned().collect()
    }
}