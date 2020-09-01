use crate::parser::{ScopeImpl, ScopeImplType, Statement, Expression};

use crate::tokenizer::{
    Type,
    Identifier,
    Literal,
    Operator,
};
use std::str::from_utf8;

pub(crate) struct Program {
    pub header: Vec<u8>,
    pub text: Vec<u8>,
    pub ro_data: Vec<u8>,
    pub bss: Vec<u8>,
}

impl Program {
    pub(crate) fn new() -> Program {
        let mut p = Program {
            header: vec![],
            text: vec![],
            ro_data: vec![],
            bss: vec![],
        };
        p.append_text(b"section .text\n");
        p.append_bss(b"section .bss\n");
        p.append_data(b"section .data\n");
        p
    }
    pub(crate) fn compile(&mut self, scopes: Vec<ScopeImpl>) {
        let global = scopes.first().unwrap();
        for statement in &global.scope {
            match statement {
                Statement::Scoped { scoped } => {
                    match &scoped.scope_type {
                        ScopeImplType::Fn { name, external } => {
                            if *external {
                                self.append_header(b"global ");
                                self.append_header(name);
                                self.append_header(b"\n");
                            }
                            self.append_text(name);
                            self.append_text(b":\n");
                            self.compile_scope(scoped);
                            self.append_text(b"ret\n\n");
                        }
                        _ => {}
                    }
                }
                // lets are mutable
                Statement::NewLet { name, literal, v_type } => {
                    let v_name = if let Identifier::Variable(v_name) = name {
                        v_name
                    } else {
                        panic!("let name is bad");
                    };
                    match literal {
                        Some(l) => {}
                        None => {
                            let (vt, size) = if let Some(vt) = v_type {
                                vt
                            } else {
                                panic!("need type for uninitialized");
                            };
                            let v8_name = from_utf8(v_name).unwrap();
                            match vt {
                                Type::Uint8 => {
                                    self.append_bss(format!("{}: resb {}\n", v8_name, size).as_ref());
                                },
                                Type::Uint64 => {
                                    self.append_bss(format!("{}: resq {}\n", v8_name, size).as_ref());
                                }
                            }
                            self.append_bss(format!("{}_LEN equ $ - {}\n", v8_name.to_uppercase(), v8_name).as_ref());
                        }
                    }
                }
                Statement::NewConst { name, literal, v_type } => {
                    let v_name = if let Identifier::Variable(v_name) = name {
                        v_name
                    } else {
                        panic!("const name is bad");
                    };
                    match literal {
                        Literal::String(s) => {
                            let v8_name = from_utf8(v_name).unwrap();
                            let s8 = from_utf8(s).unwrap();
                            self.append_data(format!("{}: db \"{}\", 10\n", v8_name, s8).as_ref());
                            self.append_data(format!("{}_LEN equ $ - {}\n", v8_name.to_uppercase(), v8_name).as_ref());
                        }
                        _ => {}
                    };
                }
                _ => {}
            }
        };
    }

    fn compile_scope(&mut self, f: &ScopeImpl) {
        let scope = &f.scope;
        let mut scope_counter: usize = 0;
        for statement in scope {
            match statement {
                Statement::InlineAssembly { instructions } => {
                    self.append_text(instructions);
                    self.append_text(b"\n");
                }
                Statement::RegisterAssign { register, expression } => {
                    match expression {
                        Expression::Variable { var } => {
                            self.append_text(b"mov ");
                            self.append_text(register);
                            self.append_text(b", ");
                            self.append_text(var);
                            self.append_text(b"\n");
                        }
                        _ => {
                            let (data_reg, expr_code) = self.compile_expression(expression.clone());
                            self.append_text(&*expr_code);
                            let src = Program::expression_data(data_reg);
                            self.append_text(format!("mov {}, {}\n", from_utf8(register).unwrap(), src).as_ref());
                        }
                    }
                },
                // no time to optimize so it's duplicate code for now
                Statement::RegisterDerefAssign { register, expression } => {
                    match expression {
                        Expression::Variable { var } => {
                            self.append_text(b"mov [");
                            self.append_text(register);
                            self.append_text(b"], ");
                            self.append_text(var);
                            self.append_text(b"\n");
                        }
                        _ => {
                            let (data_reg, expr_code) = self.compile_expression(expression.clone());
                            self.append_text(&*expr_code);
                            let src = Program::expression_data(data_reg);
                            self.append_text(format!("mov byte [{}], {}\n", from_utf8(register).unwrap(), src).as_ref());
                        }
                    }
                }
                Statement::Call { f } => {
                    self.append_text(b"call ");
                    self.append_text(f);
                    self.append_text(b"\n");
                }
                Statement::Scoped { scoped } => {
                    match &scoped.scope_type {
                        ScopeImplType::If {
                            left, right, condition
                        } => {
                            let (left_src, left_code) = self.compile_expression(left.clone());
                            let (right_src, right_code) = self.compile_expression(right.clone());
                            self.append_text(&left_code);
                            self.append_text(&right_code);

                            self.append_text(format!("cmp {}, {}\n", Program::expression_data(left_src), Program::expression_data(right_src)).as_ref());
                            match condition {
                                Operator::NotEqual => {
                                    self.append_text(format!("je ._{}_\n", scope_counter).as_ref());
                                },
                                Operator::Equal => {
                                    self.append_text(format!("jne ._{}_\n", scope_counter).as_ref());
                                }
                                _ => {}
                            }
                            self.compile_scope(scoped);
                            self.append_text(format!("._{}_:\n", scope_counter).as_ref());
                            scope_counter += 1;
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    /*
    consider most basic case:
        +
      /   \
     rax  1
     result will be in rax, instructions will be
     add rax, 1
     sub is the same
         *
       /   \
      rax  2
     will work out to be
     lea rax, [rax*2]
     as such, for larger expressions we could just simplify something like
           +
         /   \
       rax   *
           /  \
          rbx  3
     into:
     lea rbx, [rbx*3] and
        +
      /  \
     rax rbx
     */
    fn compile_expression(&mut self, expr: Expression) -> (Expression, Vec<u8>) {
        let mut instructions: Vec<u8> = vec![];
        let expr_copy = expr.clone();
        (match expr {
            Expression::Binary { left, right, operator } => {
                let (atom_left, left_code) = self.compile_expression(*left);
                let (atom_right, right_code) = self.compile_expression(*right);
                instructions.append(&mut right_code.clone());
                instructions.append(&mut left_code.clone());
                match (atom_left, atom_right) {
                    (Expression::Number { value: v1 }, Expression::Number { value: v2 }) => {
                        // cant create asm code for 2 numbers together so just optimize
                        Expression::Number { value: Program::operator_function(operator)(v1, v2) }
                    }
                    (Expression::Register { reg }, Expression::Number { value })
                    | (Expression::Number { value }, Expression::Register { reg }) => {
                        // create code to apply an imm to a register using the op
                        instructions.append(&mut Program::operator_reg_imm(operator, &reg, value));
                        Expression::Register { reg }
                    }
                    (Expression::Register { reg: r1 }, Expression::Register { reg: r2 }) => {
                        instructions.append(&mut Program::operator_reg_reg(operator, &r1, &r2));
                        Expression::Register { reg: r1 }
                    }
                    _ => panic!("operations between registers is not supported")
                }
            }
            Expression::Number { value } => expr_copy,
            Expression::Register { reg } => expr_copy,
            _ => panic!("i am sleep deprived")
        }, instructions)
    }

    fn operator_function(operator: Operator) -> (fn(usize, usize) -> usize) {
        match operator {
            Operator::Add => |x, y| x + y,
            Operator::Multiply => |x, y| x * y,
            _ => panic!("operator not valid")
        }
    }

    fn expression_data(data: Expression) -> String {
        match data {
            Expression::Number { value } => value.to_string(),
            Expression::Register { reg } => from_utf8(&reg).unwrap().to_string(),
            _ => panic!("expression is not computable")
        }
    }

    fn operator_reg_imm(operator: Operator, register: &Vec<u8>, imm: usize) -> Vec<u8> {
        let reg = String::from_utf8(register.clone()).unwrap();
        match operator {
            Operator::Add => {
                format!("add {}, {}\n", reg, imm).into_bytes()
            }
            Operator::Multiply => {
                format!("lea {}, [{}*{}]\n", reg, reg, imm).into_bytes()
            },
            Operator::Subtract => {
                format!("sub {}, {}\n", reg, imm).into_bytes()
            }
            _ => panic!("cant gen code for register and imm")
        }
    }

    fn operator_reg_reg(operator: Operator, r1: &Vec<u8>, r2: &Vec<u8>) -> Vec<u8> {
        let reg1 = String::from_utf8(r1.clone()).unwrap();
        let reg2 = String::from_utf8(r2.clone()).unwrap();
        match operator {
            Operator::Add => {
                format!("add {}, {}\n", reg1, reg2).into_bytes()
            },
            Operator::Subtract => {
                format!("sub {}, {}\n", reg1, reg2).into_bytes()
            }
            _ => panic!("this operation is not supported from register to register")
        }
    }

    // fn is_atom(expr: Expression) -> bool {
    //     match expr {
    //         Expression::Register | Expression::SizeOf | Expression::Number => true,
    //         _ => false
    //     }
    // }

    fn append_text(&mut self, text: &[u8]) {
        self.text.extend_from_slice(text);
    }

    fn append_header(&mut self, text: &[u8]) {
        self.header.extend_from_slice(text);
    }

    fn append_data(&mut self, text: &[u8]) {
        self.ro_data.extend_from_slice(text);
    }

    fn append_bss(&mut self, text: &[u8]) {
        self.bss.extend_from_slice(text);
    }
}