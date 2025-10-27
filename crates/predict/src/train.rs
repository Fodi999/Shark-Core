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

/// Load a pack of canonical knowledge CSVs so the system can ingest foundational facts.
pub fn load_knowledge_pack() {
    use std::path::Path;
    let base_dir = "crates/predict/data/knowledge";

    // Start with the canonical list (kept for ordering), then include any additional CSVs found in the folder
    let mut files = vec![
        "crates/predict/data/knowledge/knowledge_math.csv".to_string(),
        "crates/predict/data/knowledge/knowledge_analysis.csv".to_string(),
        "crates/predict/data/knowledge/knowledge_geometry.csv".to_string(),
        "crates/predict/data/knowledge/knowledge_logic.csv".to_string(),
        "crates/predict/data/knowledge/knowledge_relations.csv".to_string(),
    ];

    if let Ok(entries) = std::fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "csv").unwrap_or(false) {
                let s = path.to_string_lossy().to_string();
                if !files.iter().any(|f| f == &s) {
                    files.push(s);
                }
            }
        }
    }

    for file in files {
        println!("[knowledge] loading {}", file);
        if Path::new(&file).exists() {
            // train_from_csv is a small demo loader that prints entries; guard against panics
            let res = std::panic::catch_unwind(|| {
                train_from_csv(&file);
            });
            if res.is_err() {
                eprintln!("⚠️ Не удалось загрузить {}: panicked during parsing", file);
            }
        } else {
            eprintln!("⚠️ Не найдено {} — пропускаем", file);
        }
    }
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

/// Load problems from CSV `question,expected` (header optional). Returns vector of pairs.
pub fn load_problems(path: &str) -> Vec<(String, String)> {
    let file = File::open(path).ok();
    let reader = file.map(BufReader::new);
    let mut out = Vec::new();
    if let Some(r) = reader {
        for (i, line) in r.lines().enumerate() {
            if let Ok(l) = line {
                if i == 0 && l.to_lowercase().contains("question") { continue; }
                let parts: Vec<&str> = l.splitn(2, ',').collect();
                if parts.len() != 2 { continue; }
                let q = parts[0].trim().trim_matches('"').to_string();
                let e = parts[1].trim().trim_matches('"').to_string();
                out.push((q, e));
            }
        }
    }
    out
}

fn normalize_answer(s: &str) -> String {
    s.trim().to_lowercase().replace(' ', "")
}

/// Evaluate problems using available heuristics and AI fallback.
/// Writes a short report to `docs/problems_report.md` and returns (successes, total).
pub fn evaluate_problems(ai: &mut crate::AI, path: &str) -> (usize, usize) {
    let problems = load_problems(path);
    let total = problems.len();
    let mut ok = 0usize;
    let mut report = String::new();
    report.push_str(&format!("Problems report — {} entries\n\n", total));

    for (i, (q, expected)) in problems.iter().enumerate() {
        report.push_str(&format!("[{}] Q: {}\n", i+1, q));
        // try exact knowledge
        let mut answer = None;
        if let Some(a) = find_answer("crates/predict/data/knowledge.csv", q) {
            answer = Some(a);
        }
        // heuristics: try to sanitize question to ASCII-only expression parts
        if answer.is_none() {
            // sanitized keeps digits, ascii letters (like x), and math operators
            let sanitized: String = q.chars().filter(|c| c.is_ascii() && (c.is_ascii_digit() || c.is_ascii_alphabetic() || 
                "+-*/=()^ .".contains(*c))).collect();
            if let Some(a) = eval_arith(q) {
                answer = Some(a);
            } else if !sanitized.is_empty() {
                if let Some(a) = eval_arith(&sanitized) { answer = Some(a); }
            }
        }
        if answer.is_none() {
            // try sanitized linear equation parsing if '=' present
            let sanitized: String = q.chars().filter(|c| c.is_ascii() && (c.is_ascii_digit() || c.is_ascii_alphabetic() || 
                "+-*/=()^ .".contains(*c))).collect();
            if sanitized.contains('=') {
                if let Some(a) = solve_linear_equation(&sanitized) { answer = Some(a); }
            }
        }
        // fallback to AI
        if answer.is_none() {
            let resp = ai.chat(q);
            answer = Some(resp);
        }

        let answer = answer.unwrap_or_else(|| "".to_string());
        report.push_str(&format!("  A: {}\n  expected: {}\n", answer, expected));
        if normalize_answer(&answer) == normalize_answer(expected) {
            report.push_str("  ✅ OK\n\n");
            ok += 1;
        } else {
            report.push_str("  ❌ MISMATCH\n\n");
            // record unknown for later automatic re-learning
            let _ = append_unknown("crates/predict/data/unknowns.csv", q, expected);
            // Also append a placeholder to knowledge.csv so the system remembers the failure
            // and will attempt to re-solve it on next runs (self-learning loop).
            // Use a sentinel answer "UNKNOWN"; avoid duplicates.
            if find_answer("crates/predict/data/knowledge.csv", q).is_none() {
                let _ = append_knowledge("crates/predict/data/knowledge.csv", q, "UNKNOWN");
                println!("[learn] добавлена новая задача в knowledge.csv для повторного изучения: {}", q);
            }
        }
    }

    let summary = format!("Summary: {}/{} solved\n", ok, total);
    report.push_str(&summary);
    let _ = fs::create_dir_all("docs");
    let _ = fs::write("docs/problems_report.md", &report);
    println!("[train] {}", summary.trim());
    (ok, total)
}

/// Append an unknown problem to CSV: question,expected,date,attempts
pub fn append_unknown(path: &str, question: &str, expected: &str) -> std::io::Result<()> {
    let dir = std::path::Path::new(path).parent().unwrap_or_else(|| std::path::Path::new("crates/predict/data"));
    std::fs::create_dir_all(dir)?;
    let exists = std::path::Path::new(path).exists();
    if !exists {
        let mut f = std::fs::File::create(path)?;
        writeln!(f, "question,expected,date,attempts")?;
    }
    let now = chrono::Utc::now().to_rfc3339();
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "\"{}\",\"{}\",\"{}\",{}", question.replace('"', "'"), expected.replace('"', "'"), now, 0)?;
    Ok(())
}

/// Load unknowns into memory as (question, expected, attempts)
pub fn load_unknowns(path: &str) -> Vec<(String, String, i32)> {
    let file = File::open(path).ok();
    let reader = file.map(BufReader::new);
    let mut out = Vec::new();
    if let Some(r) = reader {
        for (i, line) in r.lines().enumerate() {
            if let Ok(l) = line {
                if i == 0 { continue; }
                let parts: Vec<&str> = l.splitn(4, ',').collect();
                if parts.len() < 4 { continue; }
                let q = parts[0].trim().trim_matches('"').to_string();
                let e = parts[1].trim().trim_matches('"').to_string();
                let attempts = parts[3].trim().parse::<i32>().unwrap_or(0);
                out.push((q, e, attempts));
            }
        }
    }
    out
}

/// Remove an unknown entry (exact match on question and expected)
pub fn remove_unknown(path: &str, question: &str, expected: &str) -> std::io::Result<()> {
    let mut rows = Vec::new();
    if let Ok(s) = std::fs::read_to_string(path) {
        for (i, line) in s.lines().enumerate() {
            if i == 0 { rows.push(line.to_string()); continue; }
            if line.contains(question) && line.contains(expected) { continue; }
            rows.push(line.to_string());
        }
    }
    std::fs::write(path, rows.join("\n"))?;
    Ok(())
}

/// Try to relearn unknowns: return (learned, total)
pub fn try_relearn_unknowns(ai: &mut crate::AI, path: &str, accept_confirmations: usize) -> (usize, usize) {
    let unknowns = load_unknowns(path);
    let total = unknowns.len();
    let mut learned = 0usize;
    for (q, expected, _attempts) in unknowns.clone() {
        // try heuristics
        let mut answer = None;
        if let Some(a) = find_answer("crates/predict/data/knowledge.csv", &q) { answer = Some(a); }
        if answer.is_none() {
            // sanitized
            let sanitized: String = q.chars().filter(|c| c.is_ascii() && (c.is_ascii_digit() || c.is_ascii_alphabetic() || 
                "+-*/=()^ .".contains(*c))).collect();
            if let Some(a) = eval_arith(&sanitized) { answer = Some(a); }
        }
        if answer.is_none() {
            let resp1 = ai.chat(&q);
            // require N confirmations
            let mut agrees = 1usize;
            for _ in 1..accept_confirmations {
                let respn = ai.chat(&q);
                if respn == resp1 { agrees += 1; }
            }
            if agrees >= accept_confirmations { answer = Some(resp1); }
        }

        if let Some(ans) = answer {
            if normalize_answer(&ans) == normalize_answer(&expected) {
                // accept and add to knowledge
                let _ = append_knowledge("crates/predict/data/knowledge.csv", &q, &ans);
                let _ = remove_unknown(path, &q, &expected);
                learned += 1;
            } else {
                // increment attempts: rewrite unknowns with incremented attempts
                // simple approach: append a new file and overwrite
                let _ = append_unknown(path, &q, &expected);
            }
        }
    }
    (learned, total)
}

/// Append a QA pair to knowledge CSV (naive append).
pub fn append_knowledge(path: &str, question: &str, answer: &str) -> std::io::Result<()> {
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "\"{}\",\"{}\"", question.replace("\n", " "), answer.replace("\n", " "))?;
    Ok(())
}

/// Load Rust source knowledge CSV (file,description) into memory.
pub fn load_rust_knowledge(path: &str) -> Vec<(String, String)> {
    let file = File::open(path).ok();
    let reader = file.map(BufReader::new);
    let mut result = Vec::new();

    if let Some(r) = reader {
        for line in r.lines().skip(1) {
            if let Ok(l) = line {
                // split on first comma only (description may contain commas)
                let parts: Vec<&str> = l.splitn(2, ',').collect();
                if parts.len() == 2 {
                    result.push((
                        parts[0].trim_matches('"').to_string(),
                        parts[1].trim().trim_matches('"').to_string(),
                    ));
                }
            }
        }
    }
    result
}

use std::fs;
use regex::Regex;

/// Scan a source directory for `.rs` files, extract simple metrics and function names,
/// and write an updated `knowledge_rust.csv` (overwrites).
pub fn scan_src_and_update_knowledge(src_dir: &str, out_csv: &str) -> std::io::Result<()> {
    let mut rows: Vec<(String, String)> = Vec::new();
    let fn_re = Regex::new(r"fn\s+([a-zA-Z0-9_]+)").unwrap();
    let struct_re = Regex::new(r"struct\s+([A-Za-z0-9_]+)").unwrap();
    let impl_re = Regex::new(r"impl\b").unwrap();

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "rs" {
                    let fname = path.file_name().unwrap().to_string_lossy().to_string();
                    let text = fs::read_to_string(&path).unwrap_or_default();
                    let fn_names: Vec<String> = fn_re.captures_iter(&text)
                        .map(|c| c[1].to_string()).take(10).collect();
                    let structs = struct_re.captures_iter(&text).count();
                    let impls = impl_re.find_iter(&text).count();
                    let fns = fn_names.len();
                    let desc = format!("fns={} structs={} impls={} funcs=[{}]",
                        fns, structs, impls, fn_names.join(", "));
                    rows.push((fname, desc));
                }
            }
        }
    }

    // write CSV
    let mut f = OpenOptions::new().create(true).write(true).truncate(true).open(out_csv)?;
    writeln!(f, "file,description")?;
    for (file, desc) in rows {
        writeln!(f, "\"{}\",\"{}\"", file, desc.replace('"', "'"))?;
    }
    Ok(())
}

/// Auto-generate Rust knowledge CSV and a simple code tree markdown for docs.
pub fn auto_update_and_visualize_structure() {
    use std::path::Path;

    let src_dir = "crates/predict/src";
    let out_csv = "crates/predict/data/knowledge_rust.csv";
    let out_tree = "docs/code_tree.md";

    let mut csv_content = String::from("file,description\n");
    let mut tree = String::from("# 🧩 Shark-Core Code Tree\n\n```\n");

    fn analyze_file(path: &Path) -> (usize, usize, usize, Vec<String>) {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let fn_count = text.matches("fn ").count();
        let struct_count = text.matches("struct ").count();
        let impl_count = text.matches("impl ").count();
        let funcs = text
            .lines()
            .filter_map(|l| {
                let t = l.trim_start();
                if t.starts_with("fn ") {
                    t.split_whitespace().nth(1).map(|s| s.trim_end_matches('(').to_string())
                } else { None }
            })
            .take(10)
            .collect();
        (fn_count, struct_count, impl_count, funcs)
    }

    if let Ok(entries) = std::fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                let file = path.file_name().unwrap().to_string_lossy().to_string();
                let (fns, structs, impls, funcs) = analyze_file(&path);

                csv_content.push_str(&format!(
                    "\"{}\",\"fns={} structs={} impls={} funcs={:?}\"\n",
                    file, fns, structs, impls, funcs
                ));

                tree.push_str(&format!("{} (fns={}, structs={})\n", file, fns, structs));
            }
        }
    }

    tree.push_str("```\n");

    let _ = std::fs::create_dir_all("docs");
    let _ = std::fs::write(out_csv, csv_content);
    let _ = std::fs::write(out_tree, tree);

    println!("[auto-doc] обновлены {} и {}", out_csv, out_tree);
}
