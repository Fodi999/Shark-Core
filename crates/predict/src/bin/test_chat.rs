use predict::AI;

fn main() {
    // quick test runner that avoids the full chat.rs startup (self-repair, scans, etc.)
    // so we can directly inspect ai.chat behaviour on a simple prompt.
    let mut ai = AI::new("weights/model_int4.bin");
    let prompt = "привет";
    let raw = ai.chat(prompt);
    println!("> {}", prompt);
    // also print raw debug to stderr so GUI-style logs are comparable
    eprintln!("[test_chat] raw reply: {:?}", raw);
    let readable = predict::decode::decode_raw(&raw);
    println!("{}", readable);
}
