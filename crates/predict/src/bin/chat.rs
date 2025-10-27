use std::io::{self, BufRead, Write};
use predict::AI;
use predict::train::train_from_csv;

fn main() {
    // If a prompt is provided on the command line, run a single-shot chat and exit.
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    // run tiny dataset loader / trainer (demo)
    train_from_csv("crates/predict/data/knowledge.csv");

    // load AI (model + memory) once at startup
    let mut ai = AI::new("weights/model_int4.bin");

    if !args.is_empty() {
        let prompt = args.join(" ");
        // single-shot: use ai.chat which already uses memory internally
        let resp = ai.chat(&prompt);
        println!("> {}", prompt);
        println!("{}", resp);
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
                // call AI (this persists to memory inside)
                let resp = ai.chat(s);
                println!("AI: {}", resp);
                // flush to keep REPL responsive
                let _ = stdout.flush();
            }
            Err(_) => break,
        }
    }
}
