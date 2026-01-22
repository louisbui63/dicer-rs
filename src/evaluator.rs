use crate::parser::{Expr, Stmt};
use std::collections::HashMap;

use rand::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum EvArray {
    F(f64),
    A(Vec<EvArray>),
}

impl EvArray {
    fn is_true(&self) -> bool {
        if matches!(self, Self::F(f) if *f == 0.) {
            false
        } else {
            true
        }
    }
    fn stringify(&self) -> Result<String, String> {
        match self {
            EvArray::F(f) => {
                let c = char::from_u32(*f as u32);
                if let None = c {
                    return Err(format!("invalid unicode value : '{}'", *f as u32));
                }
                Ok(c.unwrap().to_string())
            }
            EvArray::A(a) => {
                let mut out = "".to_owned();
                for i in a {
                    if let EvArray::F(f) = i {
                        let c = char::from_u32(*f as u32);
                        if let None = c {
                            return Err(format!("invalid unicode value : '{}'", *f as u32));
                        }
                        out.push(c.unwrap())
                    } else {
                        return Err(format!(
                            "error : operator '~' doesn't automatically flatten"
                        ));
                    }
                }
                Ok(out)
            }
        }
    }
}

impl std::fmt::Display for EvArray {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if let Self::F(n) = self {
            write!(f, "{}", n)
        } else if let Self::A(a) = self {
            let mut out = "[".to_owned();
            for i in a {
                out = format!("{}{},", out, i);
            }
            if out.len() > 1 {
                out.pop();
            }
            out.push(']');
            write!(f, "{}", out)
        } else {
            unreachable!()
        }
    }
}

fn evaluate_expr(e: Expr, mem: &HashMap<String, EvArray>) -> Result<EvArray, String> {
    match e {
        Expr::Array(v) => {
            let mut out = vec![];
            for i in v.into_inner() {
                out.push(evaluate_expr(i, mem)?);
            }
            return Ok(EvArray::A(out));
        }
        Expr::Val(v) => return Ok(EvArray::F(v)),
        Expr::Var(v) => {
            if mem.contains_key(&v) {
                return Ok(mem.get(&v).unwrap().clone());
            } else {
                return Err(format!("Unknown variable '{}'", v));
            }
        }
        Expr::Operation(first, op, second) => match op {
            'd' => {
                return Ok(dice_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '+' => {
                return Ok(plus_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '-' => {
                return Ok(minus_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '*' => {
                return Ok(times_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '/' => {
                return Ok(divide_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '^' => {
                return Ok(power_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '<' => {
                return Ok(less_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '>' => {
                return Ok(more_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '=' => {
                return Ok(equal_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '|' => {
                return Ok(or_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '&' => {
                return Ok(and_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            '@' => {
                return Ok(at_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            'l' => {
                return Ok(keeplow_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            'h' => {
                return Ok(keephigh_op(
                    evaluate_expr(*first, mem)?,
                    evaluate_expr(*second, mem)?,
                )?)
            }
            'x' => return Ok(x_op(evaluate_expr(*first, mem)?, (*second, mem))?),
            '_' => return Ok(flatten_op(evaluate_expr(*first, mem)?)?),
            '!' => return Ok(shallow_flatten_op(evaluate_expr(*first, mem)?)?),
            's' => return Ok(sum_op(evaluate_expr(*first, mem)?)?),

            e => return Err(format!("Unknown operator '{}'", e)),
        },
        Expr::None => return Err("Unexpected parse artifact".to_owned()),
        Expr::Call(name, args) => {
            let mut parsed_args = vec![];
            for e in args.clone() {
                parsed_args.push(evaluate_expr(e, mem)?)
            }
            match &name[..] {
                "CONTAINS" => {
                    if args.clone().len() != 2 {
                        return Err(format!(
                            "invalid number of arguments in call to function '{}'",
                            name
                        ));
                    }
                    match parsed_args[0].clone() {
                        EvArray::F(_) => {
                            return Ok(EvArray::F((parsed_args[0] == parsed_args[1]) as u8 as f64))
                        }
                        EvArray::A(a) => {
                            for i in a {
                                if i == parsed_args[1] {
                                    return Ok(EvArray::F(1.));
                                }
                            }
                            return Ok(EvArray::F(0.));
                        }
                    }
                }
                "PUSH" => {
                    if args.clone().len() != 2 {
                        return Err(format!(
                            "invalid number of arguments in call to function '{}'",
                            name
                        ));
                    }
                    if let EvArray::A(mut a) = parsed_args[0].clone() {
                        a.push(parsed_args[1].clone());
                        return Ok(EvArray::A(a));
                    } else {
                        return Err(format!(
                            "first argument must be an array in call to function '{}'",
                            name
                        ));
                    }
                }
                _ => Err(format!("Unknown function : '{}'", name)),
            }
        }
    }
}

fn sum_op(operand: EvArray) -> Result<EvArray, String> {
    match operand {
        EvArray::A(a) => {
            let mut out = 0.;
            for i in a {
                if let EvArray::F(f) = i {
                    out += f
                } else {
                    return Err(format!("Impossible to sum an array containing arrays. Consider to flatten the array with '_'."));
                }
            }
            Ok(EvArray::F(out))
        }
        f => Ok(f),
    }
}
fn flatten_op(operand: EvArray) -> Result<EvArray, String> {
    match operand {
        EvArray::F(_) => Ok(operand),
        EvArray::A(a) => {
            let mut out = vec![];
            for i in a {
                match flatten_op(i)? {
                    EvArray::A(a) => out.append(&mut a.clone()),
                    f => out.push(f),
                }
            }
            Ok(EvArray::A(out))
        }
    }
}

fn shallow_flatten_op(operand: EvArray) -> Result<EvArray, String> {
    match operand {
        EvArray::F(_) => Ok(operand),
        EvArray::A(a) => {
            let mut out = vec![];
            for i in a {
                match i {
                    EvArray::A(a) => out.append(&mut a.clone()),
                    f => out.push(f),
                }
            }
            Ok(EvArray::A(out))
        }
    }
}

fn keeplow_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    match (first, second) {
        (EvArray::A(a), EvArray::F(f)) => {
            if f < 0. {
                return Err(format!("Rhs cannot be negative in call to operator 'l'"));
            }
            if f as usize > a.len() {
                return Err(format!(
                    "Rhs is to large compared to array in call to operator 'l'"
                ));
            }
            let mut flts = vec![];
            for i in a {
                if let EvArray::F(f) = i {
                    flts.push(f);
                } else {
                    return Err(format!(
                        "Array cannot include non floats in call to operator 'l'"
                    ));
                }
            }
            let mut out = vec![];

            flts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            for i in flts[0..(f as usize)].iter() {
                out.push(EvArray::F(*i));
            }
            Ok(EvArray::A(out))
        }
        _ => Err(format!(
            "Operator 'l' only accepts <array>l<number> as operands"
        )),
    }
}

fn keephigh_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    match (first, second) {
        (EvArray::A(a), EvArray::F(f)) => {
            if f < 0. {
                return Err(format!("Rhs cannot be negative in call to operator 'h'"));
            }
            if f as usize > a.len() {
                return Err(format!(
                    "Rhs is to large compared to array in call to operator 'h'"
                ));
            }
            let mut flts = vec![];
            for i in a {
                if let EvArray::F(f) = i {
                    flts.push(f);
                } else {
                    return Err(format!(
                        "Array cannot include non floats in call to operator 'h'"
                    ));
                }
            }
            let mut out = vec![];

            flts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            flts.reverse();
            for i in flts[0..(f as usize)].iter() {
                out.push(EvArray::F(*i));
            }
            Ok(EvArray::A(out))
        }
        _ => Err(format!(
            "Operator 'h' only accepts <array>l<number> as operands"
        )),
    }
}

fn x_op(first: EvArray, second: (Expr, &HashMap<String, EvArray>)) -> Result<EvArray, String> {
    match first {
        EvArray::F(f) => {
            let mut out = vec![];
            for _ in 0..(f as usize) {
                let s = evaluate_expr(second.0.clone(), second.1)?;
                out.push(s)
            }
            Ok(EvArray::A(out))
        }
        EvArray::A(a) => {
            let s = evaluate_expr(second.0, second.1)?;
            if let EvArray::F(f) = s {
                let mut out = vec![];
                for _ in 0..(f as usize) {
                    for i in a.clone() {
                        out.push(i);
                    }
                }

                Ok(EvArray::A(out))
            } else {
                Err(format!("Cannot infer duplication number from array"))
            }
        }
    }
}

fn at_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    if let EvArray::F(f) = first {
        if let EvArray::A(a) = second {
            if f < 0. {
                Err(format!("Impossible to index with a negative number"))
            } else if f as usize >= a.len() {
                Err(format!("Index larger than array length"))
            } else {
                Ok(a[f as usize].clone())
            }
        } else {
            Err(format!("Impossible to index a float"))
        }
    } else {
        Err(format!("Impossibe to index with an array"))
    }
}

fn and_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    if let EvArray::F(f) = first {
        if let EvArray::F(s) = second {
            Ok(EvArray::F(if f != 0. && s != 0. { 1. } else { 0. }))
        } else {
            Err(format!("Logical operators cannot be used on arrays"))
        }
    } else {
        Err(format!("Logical operators cannot be used on arrays"))
    }
}

fn or_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    if let EvArray::F(f) = first {
        if let EvArray::F(s) = second {
            Ok(EvArray::F(if f != 0. || s != 0. { 1. } else { 0. }))
        } else {
            Err(format!("Logical operators cannot be used on arrays"))
        }
    } else {
        Err(format!("Logical operators cannot be used on arrays"))
    }
}

fn equal_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    match (first, second) {
        (EvArray::A(_), EvArray::F(_)) | (EvArray::F(_), EvArray::A(_)) => Ok(EvArray::F(0.)),
        (EvArray::F(f), EvArray::F(s)) => Ok(EvArray::F(if f == s { 1. } else { 0. })),
        (EvArray::A(f), EvArray::A(s)) => {
            if f.len() != s.len() {
                return Ok(EvArray::F(0.));
            }
            let mut is_equal = true;
            for i in 0..f.len() {
                if let EvArray::F(a) = equal_op(f[i].clone(), s[i].clone())? {
                    if a == 0. {
                        is_equal = false;
                        break;
                    }
                }
            }
            Ok(EvArray::F(if is_equal { 1. } else { 0. }))
        }
    }
}

fn more_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    if let EvArray::F(f) = first {
        if let EvArray::F(s) = second {
            Ok(EvArray::F(if f > s { 1. } else { 0. }))
        } else {
            Err(format!("Logical operators cannot be used on arrays"))
        }
    } else {
        Err(format!("Logical operators cannot be used on arrays"))
    }
}

fn less_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    if let EvArray::F(f) = first {
        if let EvArray::F(s) = second {
            Ok(EvArray::F(if f < s { 1. } else { 0. }))
        } else {
            Err(format!("Logical operators cannot be used on arrays"))
        }
    } else {
        Err(format!("Logical operators cannot be used on arrays"))
    }
}

fn power_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    match (first, second) {
        (EvArray::F(f), EvArray::F(s)) => Ok(EvArray::F(f.powf(s))),
        (EvArray::F(f), EvArray::A(a)) => {
            let mut out = vec![];
            for i in a {
                out.push(power_op(EvArray::F(f), i)?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(a), EvArray::F(f)) => {
            let mut out = vec![];
            for i in a {
                out.push(power_op(i, EvArray::F(f))?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(f), EvArray::A(s)) => {
            if f.len() != s.len() {
                return Err(format!(
                    "Cannot use mathematical operators on differently sized arrays"
                ));
            }
            let mut out = vec![];
            for i in 0..f.len() {
                out.push(power_op(f[i].clone(), s[i].clone())?);
            }
            Ok(EvArray::A(out))
        }
    }
}

fn divide_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    match (first, second) {
        (EvArray::F(f), EvArray::F(s)) => {
            if s == 0. {
                Err(format!("Cannot divide by 0"))
            } else {
                Ok(EvArray::F(f / s))
            }
        }
        (EvArray::F(f), EvArray::A(a)) => {
            let mut out = vec![];
            for i in a {
                out.push(divide_op(EvArray::F(f), i)?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(a), EvArray::F(f)) => {
            let mut out = vec![];
            for i in a {
                out.push(divide_op(i, EvArray::F(f))?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(f), EvArray::A(s)) => {
            if f.len() != s.len() {
                return Err(format!(
                    "Cannot use mathematical operators on differently sized arrays"
                ));
            }
            let mut out = vec![];
            for i in 0..f.len() {
                out.push(divide_op(f[i].clone(), s[i].clone())?);
            }
            Ok(EvArray::A(out))
        }
    }
}

fn times_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    match (first, second) {
        (EvArray::F(f), EvArray::F(s)) => Ok(EvArray::F(f * s)),
        (EvArray::F(f), EvArray::A(a)) => {
            let mut out = vec![];
            for i in a {
                out.push(times_op(EvArray::F(f), i)?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(a), EvArray::F(f)) => {
            let mut out = vec![];
            for i in a {
                out.push(times_op(i, EvArray::F(f))?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(f), EvArray::A(s)) => {
            if f.len() != s.len() {
                return Err(format!(
                    "Cannot use mathematical operators on differently sized arrays"
                ));
            }
            let mut out = vec![];
            for i in 0..f.len() {
                out.push(times_op(f[i].clone(), s[i].clone())?);
            }
            Ok(EvArray::A(out))
        }
    }
}

fn minus_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    match (first, second) {
        (EvArray::F(f), EvArray::F(s)) => Ok(EvArray::F(f - s)),
        (EvArray::F(f), EvArray::A(a)) => {
            let mut out = vec![];
            for i in a {
                out.push(minus_op(EvArray::F(f), i)?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(a), EvArray::F(f)) => {
            let mut out = vec![];
            for i in a {
                out.push(minus_op(i, EvArray::F(f))?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(f), EvArray::A(s)) => {
            if f.len() != s.len() {
                return Err(format!(
                    "Cannot use mathematical operators on differently sized arrays"
                ));
            }
            let mut out = vec![];
            for i in 0..f.len() {
                out.push(minus_op(f[i].clone(), s[i].clone())?);
            }
            Ok(EvArray::A(out))
        }
    }
}

fn plus_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    match (first, second) {
        (EvArray::F(f), EvArray::F(s)) => Ok(EvArray::F(f + s)),
        (EvArray::F(f), EvArray::A(a)) => {
            let mut out = vec![];
            for i in a {
                out.push(plus_op(EvArray::F(f), i)?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(a), EvArray::F(f)) => {
            let mut out = vec![];
            for i in a {
                out.push(plus_op(i, EvArray::F(f))?);
            }
            Ok(EvArray::A(out))
        }
        (EvArray::A(f), EvArray::A(s)) => {
            if f.len() != s.len() {
                return Err(format!(
                    "Cannot use mathematical operators on differently sized arrays"
                ));
            }
            let mut out = vec![];
            for i in 0..f.len() {
                out.push(plus_op(f[i].clone(), s[i].clone())?);
            }
            Ok(EvArray::A(out))
        }
    }
}

fn dice_op(first: EvArray, second: EvArray) -> Result<EvArray, String> {
    match (first, second) {
        (EvArray::F(f), EvArray::F(s)) => {
            let mut rng = thread_rng();

            let mut out = vec![];

            for _ in 0..(f as usize) {
                let n: usize = rng.gen_range(1..=(s as usize));
                out.push(EvArray::F(n as f64));
            }

            Ok(EvArray::A(out))
        }
        (EvArray::F(f), EvArray::A(s)) => {
            let mut rng = thread_rng();

            let mut out = vec![];

            for _ in 0..(f as usize) {
                let n: usize = rng.gen_range(0..(s.len() as usize));
                out.push(s[n].clone());
            }

            Ok(EvArray::A(out))
        }
        (EvArray::A(_), _) => Err(format!(
            "Cannot infer the number of dice throws from an array"
        )),
    }
}

pub fn evaluate(t: &Vec<Stmt>, mem: &mut HashMap<String, EvArray>) -> Result<String, String> {
    let mut out: String = String::new();

    let mut i = 0;

    while i < t.len() {
        let c = t[i].clone();
        match c {
            Stmt::Bind(v, val) => {
                mem.insert(v, evaluate_expr(val, &mem)?);
            }
            Stmt::Out(e) => {
                let o = evaluate_expr(e, &mem)?;
                out = format!("{}{}\n", out, o);
            }
            Stmt::StringOut(e) => {
                let o = evaluate_expr(e, &mem)?;
                out = format!("{}{}\n", out, o.stringify()?);
            }
            Stmt::Condition(e, Some(ife), el) => {
                if evaluate_expr(e, &mem)?.is_true() {
                    out = format!("{}{}", out, evaluate(&ife, mem)?);
                } else if let Some(els) = el {
                    out = format!("{}{}", out, evaluate(&els, mem)?);
                }
            }
            Stmt::Condition(_, _, _) => {
                return Err(format!("Error : malformed condition at index {}", i));
            }
            Stmt::While(e, Some(bod)) => {
                while evaluate_expr(e.clone(), &mem)?.is_true() {
                    out = format!("{}{}", out, evaluate(&bod, mem)?);
                }
            }
            Stmt::While(_, _) => {
                return Err(format!("Error : malformed while loop at index {}", i));
            }
            Stmt::For(Some(v), e, Some(bod)) => {
                let es = evaluate_expr(e, mem)?;

                if let EvArray::F(_) = es {
                    mem.insert(v, es);
                    out = format!("{}{}", out, evaluate(&bod, mem)?);
                } else if let EvArray::A(a) = es {
                    for i in a {
                        mem.insert(v.clone(), i);
                        out = format!("{}{}", out, evaluate(&bod, mem)?);
                    }
                }
            }
            Stmt::For(_, _, _) => {
                return Err(format!("Error : malformed for loop at index {}", i));
            }
            Stmt::None => {
                return Err(format!("Error : unexpected parser artifact at index {}", i));
            }
        }
        i += 1;
    }

    return Ok(out);
}
