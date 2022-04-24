use std::num::ParseFloatError;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Control {
    If,
    Else,
    While,
    For,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Token {
    Number(f64),
    Operator(char),
    Variable(char),
    Control(Control),
    Output,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBraces,
    RBraces,
    Comma,
    None,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Array(Array),
    Val(f64),
    Var(char),
    Operation(Box<Expr>, char, Box<Expr>),
    None,
}

impl Expr {
    fn add_f(&self, n: f64) -> Result<Expr, String> {
        match self {
            Expr::Operation(a, b, e) => Ok(Self::Operation(
                Box::new(*a.clone()),
                *b,
                Box::new(e.add_f(n)?),
            )),
            Expr::Val(_) => Err(format!("Invalid token in expression : 'Number({})'", n)),
            Expr::Var(_) => Err(format!("Invalid token in expression : 'Number({})'", n)),
            Expr::None => Ok(Self::Val(n)),
            Expr::Array(_) => Err(format!("Invalid token in expression : 'Number({})'", n)),
        }
    }
    fn add_arr(&self, a: Array) -> Result<Expr, String> {
        match self {
            Expr::Operation(w, b, e) => Ok(Self::Operation(
                Box::new(*w.clone()),
                *b,
                Box::new(e.add_arr(a)?),
            )),
            Expr::Val(_) => Err(format!("Invalid token in expression : 'Arr({:?})'", a)),
            Expr::Var(_) => Err(format!("Invalid token in expression : 'Arr({:?})'", a)),
            Expr::None => Ok(Self::Array(a)),
            Expr::Array(_) => Err(format!("Invalid token in expression : 'Arr({:?})'", a)),
        }
    }
    fn add_op(&self, o: char) -> Result<Expr, String> {
        match self {
            Expr::Val(v) => Ok(Self::Operation(
                Box::new(Self::Val(*v)),
                o,
                Box::new(Self::None),
            )),
            Expr::Operation(a, b, e) => {
                if get_precedence(*b) > get_precedence(o) {
                    Ok(Self::Operation(
                        Box::new(*a.clone()),
                        *b,
                        Box::new(e.add_op(o)?),
                    ))
                } else {
                    Ok(Self::Operation(
                        Box::new(self.clone()),
                        o,
                        Box::new(Self::None),
                    ))
                }
            }
            Expr::None => Err(format!("Invalid token in expression : 'Operator({})'", 0)),
            Expr::Var(v) => Ok(Self::Operation(
                Box::new(Self::Var(*v)),
                o,
                Box::new(Self::None),
            )),
            Expr::Array(a) => Ok(Self::Operation(
                Box::new(Self::Array(a.clone())),
                o,
                Box::new(Self::None),
            )),
        }
    }
    fn add_var(&self, v: char) -> Result<Expr, String> {
        match self {
            Expr::Val(_) => Err(format!("Invalid token in expression : 'Variable({})'", v)),
            Expr::Var(_) => Err(format!("Invalid token in expression : 'Variable({})'", v)),
            Expr::Operation(a, b, e) => Ok(Self::Operation(
                Box::new(*a.clone()),
                *b,
                Box::new(e.add_var(v)?),
            )),
            Expr::None => Ok(Self::Var(v)),
            Expr::Array(_) => Err(format!("Invalid token in expression : 'Variable({})'", v)),
        }
    }

    fn add_expr(&self, i: Expr) -> Result<Expr, String> {
        match self {
            Expr::Val(_) => Err(format!("Invalid token in expression : 'Expr({:?})'", i)),
            Expr::Var(_) => Err(format!("Invalid token in expression : 'Expr({:?})'", i)),
            Expr::Operation(a, b, e) => Ok(Self::Operation(
                Box::new(*a.clone()),
                *b,
                Box::new(e.add_expr(i)?),
            )),
            Expr::None => Ok(i),
            Expr::Array(_) => Err(format!("Invalid token in expression : 'Expr({:?})'", i)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Bind(char, Expr),
    Out(Expr),
    Condition(Expr, Option<Vec<Stmt>>, Option<Vec<Stmt>>),
    While(Expr, Option<Vec<Stmt>>),
    For(Option<char>, Expr, Option<Vec<Stmt>>),
    None,
}

#[derive(Clone, Debug)]
pub struct Array(Vec<Expr>);

impl Array {
    pub fn into_inner(&self) -> Vec<Expr> {
        return self.0.clone();
    }
}

fn is_operator(c: char) -> bool {
    return match c {
        'd' | '+' | '-' | '*' | '/' | '^' | '<' | '>' | '=' | '|' | '&' | '@' | 'x' | 'l' | 'h' => {
            true
        }
        _ => false,
    };
}

fn get_precedence(c: char) -> usize {
    return match c {
        'x' | 'l' | 'h' => 6,
        '+' | '-' => 5,
        '*' | '/' => 4,
        '^' => 3,
        '@' => 2,
        'd' => 1,
        '<' | '>' => 7,
        '=' => 8,
        _ => unreachable!(),
    };
}

pub fn tokenize(s: String) -> Result<Vec<Token>, ParseFloatError> {
    let chars = s.chars();
    let mut out = vec![];

    let mut c_token = Token::None;
    for c in chars {
        match c {
            '0'..='9' | '.' => {
                if let Token::Number(n) = c_token {
                    c_token = Token::Number(std::str::FromStr::from_str(
                        &([n.to_string(), c.to_string()].concat())[..],
                    )?)
                } else {
                    if Token::None != c_token {
                        out.push(c_token);
                    }
                    c_token = Token::Number(std::str::FromStr::from_str(&c.to_string()[..])?);
                }
            }
            '(' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::LParen;
            }
            ')' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::RParen;
            }
            '[' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::LBracket;
            }
            ']' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::RBracket;
            }
            '{' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::LBraces;
            }
            '}' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::RBraces;
            }
            ',' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::Comma;
            }
            ' ' => {}
            'i' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::Control(Control::If);
            }
            'e' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::Control(Control::Else);
            }
            'f' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::Control(Control::For);
            }
            'w' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::Control(Control::While);
            }
            '$' => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                c_token = Token::Output;
            }
            c => {
                if Token::None != c_token {
                    out.push(c_token);
                }
                if c.is_uppercase() {
                    c_token = Token::Variable(c);
                } else if is_operator(c) {
                    c_token = Token::Operator(c);
                }
            }
        }
    }

    out.push(c_token);

    return Ok(out);
}

fn parse_parenthesis(t: &Vec<Token>, i: &mut usize) -> Result<Expr, String> {
    let mut out = Expr::None;

    while *i < t.len() {
        match t[*i] {
            Token::Number(n) => out = out.add_f(n)?,
            Token::Variable(v) => out = out.add_var(v)?,
            Token::Operator(o) => out = out.add_op(o)?,
            Token::LParen => {
                *i += 1;
                out = out.add_expr(parse_parenthesis(t, i)?)?
            }
            Token::RParen => break,
            Token::LBracket => {
                *i += 1;
                out = out.add_arr(parse_array(t, i)?)?
            }
            e => return Err(format!("Invalid token in argument {:?} at index {}", e, i)),
        }
        *i += 1;
    }

    return Ok(out);
}

pub fn parse_array(t: &Vec<Token>, i: &mut usize) -> Result<Array, String> {
    let mut out = Array(vec![]);

    let mut current_expr = Expr::None;

    while *i < t.len() {
        match t[*i] {
            Token::Number(n) => {
                current_expr = current_expr.add_f(n)?;
            }
            Token::Operator(o) => {
                current_expr = current_expr.add_op(o)?;
            }
            Token::Variable(v) => {
                current_expr = current_expr.add_var(v)?;
            }
            Token::LParen => {
                *i += 1;
                current_expr = current_expr.add_expr(parse_parenthesis(t, i)?)?;
            }
            Token::LBracket => {
                *i += 1;
                current_expr = current_expr.add_arr(parse_array(t, i)?)?;
            }

            Token::RBracket => {
                if let Expr::None = current_expr {
                } else {
                    out.0.push(current_expr.clone())
                }
                break;
            }
            Token::Comma => {
                if let Expr::None = current_expr {
                    return Err(format!(
                        "Invalid token in array : {:?} at index {}",
                        t[*i], i
                    ));
                } else {
                    out.0.push(current_expr.clone());
                    current_expr = Expr::None;
                }
            }
            _ => {
                return Err(format!(
                    "Invalid token in array : {:?} at index {}",
                    t[*i], i
                ))
            }
        }
        *i += 1;
    }

    return Ok(out);
}

pub fn parse(t: &Vec<Token>, i: &mut usize) -> Result<Vec<Stmt>, String> {
    let mut out = vec![];

    let mut current_stmt = Stmt::None;

    while *i < t.len() {
        match t[*i] {
            Token::Number(n) => {
                if let Stmt::Bind(u, expr) = current_stmt.clone() {
                    current_stmt = Stmt::Bind(u, expr.add_f(n)?)
                } else if let Stmt::Out(expr) = current_stmt.clone() {
                    current_stmt = Stmt::Out(expr.add_f(n)?)
                } else if let Stmt::Condition(expr, None, None) = current_stmt.clone() {
                    current_stmt = Stmt::Condition(expr.add_f(n)?, None, None)
                } else if let Stmt::While(expr, None) = current_stmt.clone() {
                    current_stmt = Stmt::While(expr.add_f(n)?, None)
                } else if let Stmt::For(Some(v), expr, None) = current_stmt.clone() {
                    current_stmt = Stmt::For(Some(v), expr.add_f(n)?, None)
                } else {
                    return Err(format!("Invalid Token 'Number({})' at index {}", n, i));
                }
            }
            Token::Operator(o) => {
                if let Stmt::Bind(u, expr) = current_stmt.clone() {
                    if let Expr::None = expr {
                        if o != '=' {
                            return Err(format!("Expected operator '=' at index {}", i));
                        }
                    } else {
                        current_stmt = Stmt::Bind(u, expr.add_op(o)?)
                    }
                } else if let Stmt::Out(expr) = current_stmt.clone() {
                    current_stmt = Stmt::Out(expr.add_op(o)?)
                } else if let Stmt::Condition(expr, None, None) = current_stmt.clone() {
                    current_stmt = Stmt::Condition(expr.add_op(o)?, None, None)
                } else if let Stmt::While(expr, None) = current_stmt.clone() {
                    current_stmt = Stmt::While(expr.add_op(o)?, None)
                } else if let Stmt::For(Some(v), expr, None) = current_stmt.clone() {
                    current_stmt = Stmt::For(Some(v), expr.add_op(o)?, None)
                } else {
                    return Err("Invalid Token".to_owned());
                }
            }
            Token::Variable(v) => {
                if let Stmt::None = current_stmt {
                    current_stmt = Stmt::Bind(v, Expr::None);
                } else if let Stmt::Bind(u, expr) = current_stmt.clone() {
                    match expr.add_var(v) {
                        Ok(e) => current_stmt = Stmt::Bind(u, e),
                        Err(_) => {
                            out.push(current_stmt);
                            current_stmt = Stmt::Bind(v, Expr::None);
                        }
                    }
                } else if let Stmt::Out(expr) = current_stmt.clone() {
                    match expr.add_var(v) {
                        Ok(e) => current_stmt = Stmt::Out(e),
                        Err(_) => {
                            out.push(current_stmt);
                            current_stmt = Stmt::Bind(v, Expr::None);
                        }
                    }
                } else if let Stmt::Condition(expr, None, None) = current_stmt.clone() {
                    current_stmt = Stmt::Condition(expr.add_var(v)?, None, None)
                } else if let Stmt::While(expr, None) = current_stmt.clone() {
                    current_stmt = Stmt::While(expr.add_var(v)?, None)
                } else if let Stmt::For(None, Expr::None, None) = current_stmt.clone() {
                    current_stmt = Stmt::For(Some(v), Expr::None, None);
                } else if let Stmt::For(Some(v), e, None) = current_stmt.clone() {
                    current_stmt = Stmt::For(Some(v), e.add_var(v)?, None);
                } else {
                    return Err("Invalid Token".to_owned());
                }
            }
            Token::Control(control) => {
                if let Control::Else = control {
                    if let Stmt::Condition(_, _, _) = current_stmt.clone() {
                        // we should store the fact that the else was there
                    } else {
                        return Err("Invalid Token".to_owned());
                    }
                } else if let Control::If = control {
                    if let Stmt::None = current_stmt {
                    } else {
                        out.push(current_stmt.clone());
                    }

                    current_stmt = Stmt::Condition(Expr::None, None, None);
                } else if let Control::While = control {
                    if let Stmt::None = current_stmt {
                    } else {
                        out.push(current_stmt.clone());
                    }

                    current_stmt = Stmt::While(Expr::None, None);
                } else if let Control::For = control {
                    if let Stmt::None = current_stmt {
                    } else {
                        out.push(current_stmt.clone())
                    }

                    current_stmt = Stmt::For(None, Expr::None, None);
                }
            }
            Token::Output => {
                if let Stmt::None = current_stmt {
                } else {
                    out.push(current_stmt.clone());
                }
                current_stmt = Stmt::Out(Expr::None);
            }
            Token::LParen => {
                *i += 1;
                if let Stmt::Bind(u, expr) = current_stmt.clone() {
                    current_stmt = Stmt::Bind(u, expr.add_expr(parse_parenthesis(t, i)?)?)
                } else if let Stmt::Out(expr) = current_stmt.clone() {
                    current_stmt = Stmt::Out(expr.add_expr(parse_parenthesis(t, i)?)?)
                } else if let Stmt::Condition(expr, None, None) = current_stmt.clone() {
                    current_stmt =
                        Stmt::Condition(expr.add_expr(parse_parenthesis(t, i)?)?, None, None)
                } else if let Stmt::While(expr, None) = current_stmt.clone() {
                    current_stmt = Stmt::While(expr.add_expr(parse_parenthesis(t, i)?)?, None)
                } else if let Stmt::For(Some(v), expr, None) = current_stmt.clone() {
                    current_stmt =
                        Stmt::For(Some(v), expr.add_expr(parse_parenthesis(t, i)?)?, None)
                } else {
                    return Err(format!("Invalid Token '(' at index {}", i));
                }
            }
            Token::RParen => return Err(format!("Invalid Token ')' at index {}", i)),
            Token::LBracket => {
                *i += 1;
                if let Stmt::Bind(u, expr) = current_stmt.clone() {
                    current_stmt = Stmt::Bind(u, expr.add_arr(parse_array(t, i)?)?)
                } else if let Stmt::Out(expr) = current_stmt.clone() {
                    current_stmt = Stmt::Out(expr.add_arr(parse_array(t, i)?)?)
                } else if let Stmt::Condition(expr, None, None) = current_stmt.clone() {
                    current_stmt = Stmt::Condition(expr.add_arr(parse_array(t, i)?)?, None, None)
                } else if let Stmt::While(expr, None) = current_stmt.clone() {
                    current_stmt = Stmt::While(expr.add_arr(parse_array(t, i)?)?, None)
                } else if let Stmt::For(Some(v), expr, None) = current_stmt.clone() {
                    current_stmt = Stmt::For(Some(v), expr.add_arr(parse_array(t, i)?)?, None)
                } else {
                    return Err(format!("Invalid Token '[' at index {}", i));
                }
            }
            Token::RBracket => return Err(format!("Invalid Token ']' at index {}", i)),
            Token::LBraces => {
                *i += 1;
                match current_stmt.clone() {
                    Stmt::Condition(e, Some(v), None) => {
                        current_stmt = Stmt::Condition(e, Some(v), Some(parse(t, i)?))
                    }

                    Stmt::Condition(e, None, None) => {
                        current_stmt = Stmt::Condition(e, Some(parse(t, i)?), None)
                    }
                    Stmt::While(e, None) => current_stmt = Stmt::While(e, Some(parse(t, i)?)),
                    Stmt::For(v, e, None) => current_stmt = Stmt::For(v, e, Some(parse(t, i)?)),
                    _ => return Err(format!("Invalid Token '{{' at index {}", i)),
                }
            }
            Token::RBraces => {
                break;
            }
            Token::Comma => return Err(format!("Invalid Token ',' at index {}", i)),
            Token::None => return Err(format!("Invalid Token 'None' at index {}", i)),
        }

        *i += 1;
    }

    if let Stmt::None = current_stmt {
    } else {
        out.push(current_stmt.clone());
    }
    return Ok(out);
}
