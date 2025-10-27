use std::io::{self, BufRead, Write};
use predict::AI;
use predict::scientist;
use predict::reasoner::Reasoner;
use predict::train::{train_from_csv, load_knowledge_pack, find_answer, eval_arith, solve_linear_equation, append_knowledge, load_rust_knowledge, scan_src_and_update_knowledge, auto_update_and_visualize_structure, evaluate_problems};
use predict::knowledge_env::{expand_knowledge_environment, merge_knowledge_sources, auto_expand_on_new_topic, detect_knowledge_gap};
use predict::self_repair::self_repair;

fn main() {
    // If a prompt is provided on the command line, run a single-shot chat and exit.
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    // Self-repair: restore missing/ corrupted critical modules before other startup steps
    self_repair(); // <- automatically repairs missing code and writes docs/self_fix.log

    // Ensure the knowledge environment exists and seed topic files if needed
    let topics = ["math", "analysis", "geometry", "logic", "science"];
    if let Err(e) = expand_knowledge_environment(&topics) {
        eprintln!("⚠️ Не удалось расширить окружение знаний: {}", e);
    }

    // Merge per-topic knowledge into the central knowledge.csv so loader can read it
    if let Err(e) = merge_knowledge_sources() {
        eprintln!("⚠️ Ошибка при объединении знаний: {}", e);
    }

    // Load the canonical knowledge pack (math, analysis, geometry, logic, relations)
    load_knowledge_pack();

    // Auto-scan source and update docs + CSV, then run tiny dataset loader / trainer (demo)
    auto_update_and_visualize_structure(); // performs automatic scan and writes docs/code_tree.md
    train_from_csv("crates/predict/data/knowledge.csv");
    // legacy: also ensure the CSV is up-to-date (no-op if auto-update already ran)
    let _ = scan_src_and_update_knowledge("crates/predict/src", "crates/predict/data/knowledge_rust.csv");

    // --- Deepen research from science memory (pick most curious formulas)
    let science_mem = scientist::load_science_memory();
    if !science_mem.is_empty() {
        // sort by curiosity desc (index 3 is curiosity in (name,formula,mse,curiosity))
        let mut mem = science_mem.clone();
        mem.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));
        // pick top 2 by curiosity to avoid long startup
        let top_n = mem.iter().take(2).cloned().collect::<Vec<_>>();
        use std::hash::{Hasher, Hash};
        use std::collections::hash_map::DefaultHasher;
        for (_name, formula, _mse, curiosity) in top_n {
            // derive a reproducible seed from formula text so evolve starts near same region
            let mut hasher = DefaultHasher::new();
            formula.hash(&mut hasher);
            let seed = hasher.finish();
            println!("[science] углублённый поиск от '{}' (curiosity={:.4})...", formula, curiosity);
            // run a deeper evolve (fewer gens if you want faster)
            let (best, fit) = scientist::evolve_symbolic(seed, 200, 60);
            println!("[science] найдено: {:?} (MSE={:.4})", best, fit);
        }
    }

    // load AI (model + memory) once at startup
    let mut ai = AI::new("weights/model_int4.bin");

    // Try to relearn unknowns from previous runs (require 2 confirmations by default)
    let (learned, total_unknowns) = predict::train::try_relearn_unknowns(&mut ai, "crates/predict/data/unknowns.csv", 2);
    if total_unknowns > 0 {
        println!("[train] relearnt {}/{} unknowns", learned, total_unknowns);
    }

    if !args.is_empty() {
        let prompt = args.join(" ");
        // Load Rust self-knowledge and handle structure/code queries
        let rust_knowledge = load_rust_knowledge("crates/predict/data/knowledge_rust.csv");
        // Detect knowledge gaps and auto-expand topic files if needed
        if let Some(topic) = detect_knowledge_gap(&prompt) {
            let _ = auto_expand_on_new_topic(&topic);
            // After expansion, merge sources so new file is visible to loaders
            let _ = merge_knowledge_sources();
        }

        // Reasoner trigger: if user asks to explain/simplify or requests an integral, run the reasoner first
        if prompt.to_lowercase().contains("упрост") || prompt.to_lowercase().contains("объясн") || prompt.to_lowercase().contains("рассужд") || prompt.to_lowercase().contains("интеграл") {
            let (ans, reasoning) = Reasoner::explain(&prompt);
            println!("> {}", prompt);
            println!("🧠 Ответ: {}", ans);
            println!("📜 Рассуждение:\n{}", reasoning);
            return;
        }
        if prompt.contains("структура") || prompt.contains("код") {
            println!("🧩 Shark-Core состоит из следующих модулей:");
            for (file, desc) in rust_knowledge {
                println!("• {} — {}", file, desc);
            }
            return;
        }

        // Command: trigger symbolic exploration
        if prompt.to_lowercase().contains("исслед") {
            let (best, fit) = scientist::evolve_symbolic(42, 300, 50);
            let formula = format!("{:?}", best);
            let curiosity = scientist::curiosity_from_mse(fit);
            println!("> {}", prompt);
            println!("🧠 Я нашёл новую закономерность: {}\nMSE = {:.4} — любознательность={:.4} ✅", formula, fit, curiosity);
            return;
        }

        // Command: evaluate problems dataset
        if prompt.to_lowercase().contains("проверь задачи") {
            println!("> {}", prompt);
            let (ok, total) = evaluate_problems(&mut ai, "crates/predict/data/problems.csv");
            println!("[train] problems scored: {}/{}\nДоклад: docs/problems_report.md", ok, total);
            return;
        }

        // Check knowledge base first
        if let Some(answer) = find_answer("crates/predict/data/knowledge.csv", &prompt) {
            println!("> {}", prompt);
            println!("🧠 Из знаний: {}", answer);
            // persist to memory
            let _ = ai.memory.save_dialog(&prompt, &answer);
            return;
        }

        // Try to compute arithmetic expression
        if let Some(ans) = eval_arith(&prompt) {
            println!("> {}", prompt);
            println!("🧠 Вычислено: {}", ans);
            let _ = append_knowledge("crates/predict/data/knowledge.csv", &prompt, &ans);
            let _ = ai.memory.save_dialog(&prompt, &ans);
            return;
        }

        // Try to solve simple linear equation
        if let Some(ans) = solve_linear_equation(&prompt) {
            println!("> {}", prompt);
            println!("🧠 Решено: {}", ans);
            let _ = append_knowledge("crates/predict/data/knowledge.csv", &prompt, &ans);
            let _ = ai.memory.save_dialog(&prompt, &ans);
            return;
        }

        // single-shot: use ai.chat which returns raw output; decode for presentation
        let raw = ai.chat(&prompt);
        let readable = predict::decode::decode_raw(&raw);
        println!("> {}", prompt);
        println!("🧠 Ответ: {}", readable);
        return;
    }

    // Interactive REPL
    println!("Interactive chat — введите 'quit' или Ctrl-D для выхода");
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        match line {
            Ok(s) => {
                let s = s.trim();
                if s.is_empty() {
                    continue;
                }
                if s.eq_ignore_ascii_case("quit") || s.eq_ignore_ascii_case("exit") {
                    println!("Bye");
                    break;
                }
                // Load Rust self-knowledge and handle structure/code queries
                let rust_knowledge = load_rust_knowledge("crates/predict/data/knowledge_rust.csv");
                // Reasoner trigger in REPL
                // Detect knowledge gaps and auto-expand topic files if needed
                if let Some(topic) = detect_knowledge_gap(s) {
                    let _ = auto_expand_on_new_topic(&topic);
                    let _ = merge_knowledge_sources();
                }
                // Reasoner trigger in REPL (include integrals)
                if s.to_lowercase().contains("упрост") || s.to_lowercase().contains("объясн") || s.to_lowercase().contains("рассужд") || s.to_lowercase().contains("интеграл") {
                    let (ans, reasoning) = Reasoner::explain(s);
                    println!("🧠 Ответ: {}", ans);
                    println!("📜 Рассуждение:\n{}", reasoning);
                    continue;
                }
                if s.contains("структура") || s.contains("код") {
                    println!("🧩 Shark-Core состоит из следующих модулей:");
                    for (file, desc) in rust_knowledge {
                        println!("• {} — {}", file, desc);
                    }
                    continue;
                }

                // Command: trigger symbolic exploration
                if s.to_lowercase().contains("исслед") {
                    let (best, fit) = scientist::evolve_symbolic(42, 300, 50);
                    let formula = format!("{:?}", best);
                    let curiosity = scientist::curiosity_from_mse(fit);
                    println!("🧠 Я нашёл новую закономерность: {}\nMSE = {:.4} — любознательность={:.4} ✅", formula, fit, curiosity);
                    continue;
                }

                // Command: evaluate problems dataset
                if s.to_lowercase().contains("проверь задачи") {
                    let (ok, total) = evaluate_problems(&mut ai, "crates/predict/data/problems.csv");
                    println!("[train] problems scored: {}/{} — доклад в docs/problems_report.md", ok, total);
                    continue;
                }

                // Check knowledge base first
                if let Some(answer) = find_answer("crates/predict/data/knowledge.csv", s) {
                    println!("🧠 Из знаний: {}", answer);
                    let _ = ai.memory.save_dialog(s, &answer);
                    continue;
                }
                // Try compute arith
                if let Some(ans) = eval_arith(s) {
                    println!("🧠 Вычислено: {}", ans);
                    let _ = append_knowledge("crates/predict/data/knowledge.csv", s, &ans);
                    let _ = ai.memory.save_dialog(s, &ans);
                    continue;
                }
                // Try linear eq
                if let Some(ans) = solve_linear_equation(s) {
                    println!("🧠 Решено: {}", ans);
                    let _ = append_knowledge("crates/predict/data/knowledge.csv", s, &ans);
                    let _ = ai.memory.save_dialog(s, &ans);
                    continue;
                }

                // call AI (this persists to memory inside)
                let raw = ai.chat(s);
                let readable = predict::decode::decode_raw(&raw);
                println!("AI: {}", readable);
                // flush to keep REPL responsive
                let _ = stdout.flush();
            }
            Err(_) => break,
        }
    }
}
