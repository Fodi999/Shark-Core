#![forbid(unsafe_code)]

use std::fs::File;
use std::io::{BufRead, BufReader};

/// Very small "training" loader that reads a CSV of input→output pairs and prints them.
/// This is intentionally tiny and side-effecting for demo purposes.
pub fn train_from_csv(path: &str) {
    let file = File::open(path).expect("no knowledge.csv found");
    let reader = BufReader::new(file);

    println!("[train] loading dataset from {path}");
    for (i, line) in reader.lines().enumerate().skip(1) {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 2 { continue; }
        let input = parts[0].trim_matches('"');
        let output = parts[1].trim_matches('"');
        println!("[{i}] Q: {input} → A: {output}");
    }
    println!("[train] dataset ready.");
}

/// Find an exact answer for `question` in a CSV of input,output pairs.
/// Returns `Some(output)` if a matching input row is found.
pub fn find_answer(path: &str, question: &str) -> Option<String> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines().skip(1) {
        if let Ok(l) = line {
            let parts: Vec<&str> = l.split(',').collect();
            if parts.len() != 2 { continue; }
            let input = parts[0].trim_matches('"');
            let output = parts[1].trim_matches('"');
            if input == question {
                return Some(output.to_string());
            }
        }
    }
    None
}

/// Try to evaluate a simple arithmetic expression (supports + - * / and parentheses).
pub fn eval_arith(expr: &str) -> Option<String> {
    // Shunting-yard to RPN, then evaluate
    #[derive(Debug)]
    enum Token {
        Num(f64),
        Op(char),
        LParen,
        RParen,
    }

    fn tokenize(s: &str) -> Option<Vec<Token>> {
        let mut tokens = Vec::new();
        let mut i = 0usize;
        let chars: Vec<char> = s.chars().collect();
        while i < chars.len() {
            let c = chars[i];
            if c.is_whitespace() { i += 1; continue; }
            if c.is_ascii_digit() || c == '.' {
                let mut j = i+1;
                while j < chars.len() && (chars[j].is_ascii_digit() || chars[j]=='.') { j+=1; }
                let num: String = chars[i..j].iter().collect();
                if let Ok(v) = num.parse::<f64>() {
                    tokens.push(Token::Num(v));
                } else { return None; }
                i = j;
                continue;
            }
            if c == '+' || c == '-' || c == '*' || c == '/' {
                tokens.push(Token::Op(c));
                i += 1;
                continue;
            }
            if c == '(' { tokens.push(Token::LParen); i += 1; continue; }
            if c == ')' { tokens.push(Token::RParen); i += 1; continue; }
            // unsupported char
            return None;
        }
        Some(tokens)
    }

    fn prec(op: char) -> i32 {
        match op { '+'|'-' => 1, '*'|'/' => 2, _ => 0 }
    }

    let toks = tokenize(expr)?;
    // to RPN
    let mut out: Vec<Token> = Vec::new();
    let mut ops: Vec<char> = Vec::new();
    for t in toks {
        match t {
            Token::Num(_) => out.push(t),
            Token::Op(op) => {
                while let Some(&top) = ops.last() {
                    if top=='(' { break; }
                    if prec(top) >= prec(op) {
                        out.push(Token::Op(top)); ops.pop();
                    } else { break; }
                }
                ops.push(op);
            }
            Token::LParen => ops.push('('),
            Token::RParen => {
                while let Some(top) = ops.pop() {
                    if top == '(' { break; }
                    out.push(Token::Op(top));
                }
            }
        }
    }
    while let Some(op) = ops.pop() { out.push(Token::Op(op)); }

    // eval RPN
    let mut stack: Vec<f64> = Vec::new();
    for t in out {
        match t {
            Token::Num(v) => stack.push(v),
            Token::Op(op) => {
                if stack.len() < 2 { return None; }
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                let r = match op {
                    '+' => a + b,
                    '-' => a - b,
                    '*' => a * b,
                    '/' => a / b,
                    _ => return None,
                };
                stack.push(r);
            }
            _ => return None,
        }
    }
    if stack.len() != 1 { return None; }
    let v = stack[0];
    if (v - v.round()).abs() < 1e-9 {
        Some(format!("{}", v.round() as i64))
    } else {
        Some(format!("{}", v))
    }
}

/// Try to solve simple linear equations with single variable `x`, e.g. "2x + 3 = 7".
pub fn solve_linear_equation(eq: &str) -> Option<String> {
    let parts: Vec<&str> = eq.split('=').collect();
    if parts.len() != 2 { return None; }
    let left = parts[0].replace(' ', "");
    let right = parts[1].replace(' ', "");

    fn coef_and_const(side: &str) -> Option<(f64, f64)> {
        // split into + / - terms, keeping sign
        let mut coef = 0.0f64;
        let mut cons = 0.0f64;
        let mut i = 0usize;
        let s = side;
        while i < s.len() {
            // determine sign
            let mut sign = 1.0f64;
            if &s[i..i+1] == "+" { sign = 1.0; i += 1; }
            else if &s[i..i+1] == "-" { sign = -1.0; i += 1; }
            // read term until next + or -
            let mut j = i;
            while j < s.len() && &s[j..j+1] != "+" && &s[j..j+1] != "-" { j+=1; }
            let term = &s[i..j];
            if term.contains('x') {
                // coefficient
                let t = term.replace("x", "");
                let c = if t.is_empty() { 1.0 } else if t=="+" {1.0} else if t=="-" {-1.0} else { t.parse::<f64>().ok()? };
                coef += sign * c;
            } else if !term.is_empty() {
                let v = term.parse::<f64>().ok()?;
                cons += sign * v;
            }
            i = j;
        }
        Some((coef, cons))
    }

    let (a1, b1) = coef_and_const(&left)?; // a1*x + b1
    let (a2, b2) = coef_and_const(&right)?; // a2*x + b2
    let denom = a1 - a2;
    if denom.abs() < 1e-12 { return None; }
    let x = (b2 - b1) / denom;
    if (x - x.round()).abs() < 1e-9 { Some(format!("x = {}", x.round() as i64)) }
    else { Some(format!("x = {}", x)) }
}

use std::fs::OpenOptions;
use std::io::Write;

/// Append a QA pair to knowledge CSV (naive append).
pub fn append_knowledge(path: &str, question: &str, answer: &str) -> std::io::Result<()> {
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "\"{}\",\"{}\"", question.replace("\n", " "), answer.replace("\n", " "))?;
    Ok(())
}
