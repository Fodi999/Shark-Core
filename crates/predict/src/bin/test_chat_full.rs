use predict::{AI, train, knowledge_env};

fn main() {
    // Lightweight startup: merge knowledge sources and load knowledge pack without running self_repair
    if let Err(e) = knowledge_env::merge_knowledge_sources() {
        eprintln!("[test_chat_full] merge_knowledge_sources failed: {}", e);
    }
    train::load_knowledge_pack();
    // train_from_csv prints dataset entries and '[train] dataset ready.'
    train::train_from_csv("crates/predict/data/knowledge.csv");

    // Create AI and run a single-shot prompt
    let mut ai = AI::new("weights/model_int4.bin");
        let prompt = "почему буквы A-Z важны?";
    println!("> {}", prompt);

    // Try semantic understanding first
    if let Some(semantic_reply) = predict::interpret_question(&prompt, &ai.knowledge) {
        println!("🧠 Ответ: {}", semantic_reply);
        return;
    }

    let raw = ai.chat(prompt);
    eprintln!("[test_chat_full] raw reply: {:?}", raw);
    println!("🧠 Ответ: {}", raw);
}
