use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;
use std::path::Path;

/// Автоматически создаёт нужные папки и файлы при расширении базы знаний.
pub fn expand_knowledge_environment(topics: &[&str]) -> std::io::Result<()> {
    let base_dir = "crates/predict/data/knowledge";
    let docs_dir = "docs";
    let logs_dir = "logs";

    // 1. Ensure base directories exist
    fs::create_dir_all(base_dir)?;
    fs::create_dir_all(docs_dir)?;
    fs::create_dir_all(logs_dir)?;

    // 2. Create thematic CSVs if missing
    let mut created = Vec::new();
    for topic in topics {
        let file_path = format!("{}/knowledge_{}.csv", base_dir, topic);
        if !Path::new(&file_path).exists() {
            let mut file = fs::File::create(&file_path)?;
            // Default header; the project can update schema per-topic later
            writeln!(file, "id,topic,entry,notes,source,date")?;
            println!("🧠 [expand] создан новый файл знаний: {}", file_path);
            created.push(topic.to_string());
        }
    }

    // 3. Log the changes to docs/knowledge_log.md
    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{}/knowledge_log.md", docs_dir))?;

    let now = Utc::now().to_rfc3339();
    writeln!(log, "\n### [{}] Расширение базы знаний", now)?;
    for t in &created {
        writeln!(log, "- создан файл `knowledge_{}`", t)?;
    }

    Ok(())
}

/// Create a new topic file on demand. Logs to docs/knowledge_log.md when created.
pub fn auto_expand_on_new_topic(topic: &str) -> std::io::Result<()> {
    let base_dir = "crates/predict/data/knowledge";
    let docs_dir = "docs";
    fs::create_dir_all(base_dir)?;
    fs::create_dir_all(docs_dir)?;

    let new_path = format!("{}/knowledge_{}.csv", base_dir, topic);
    if !Path::new(&new_path).exists() {
        if let Ok(mut file) = fs::File::create(&new_path) {
            let _ = writeln!(file, "id,topic,entry,notes,source,date");
            println!("🌱 [auto-expand] создан новый файл знаний для темы: {}", topic);
        }
        let mut log = OpenOptions::new().create(true).append(true).open(format!("{}/knowledge_log.md", docs_dir))?;
        let now = Utc::now().to_rfc3339();
        writeln!(log, "\n### [{}] Автодобавление темы", now)?;
        writeln!(log, "- создан файл `knowledge_{}`", topic)?;
    }
    Ok(())
}

/// Simple heuristic detector for knowledge gaps given a user query.
/// Returns `Some(topic)` when a known gap is detected, otherwise `None`.
pub fn detect_knowledge_gap(query: &str) -> Option<String> {
    let q = query.to_lowercase();
    if q.contains("интеграл") || q.contains("производн") || q.contains("предел") {
        Some("calculus".to_string())
    } else if q.contains("матриц") || q.contains("determinant") || q.contains("матрица") {
        Some("algebra_advanced".to_string())
    } else if q.contains("квант") || q.contains("физик") || q.contains("суперпозици") {
        Some("physics".to_string())
    } else {
        None
    }
}

/// Merge simple QA-style knowledge sources (CSV files with two columns) into the central knowledge.csv
/// This is conservative: only lines with at least two comma-separated fields are merged and duplicates by question are avoided.
pub fn merge_knowledge_sources() -> std::io::Result<()> {
    let base_dir = "crates/predict/data/knowledge";
    let main_path = "crates/predict/data/knowledge.csv";
    fs::create_dir_all("crates/predict/data")?;

    // Ensure main file exists
    if !Path::new(main_path).exists() {
        let mut f = fs::File::create(main_path)?;
        writeln!(f, "question,answer")?;
    }

    // Load existing questions to avoid duplicates
    let existing = fs::read_to_string(main_path).unwrap_or_default();
    let mut seen = std::collections::HashSet::new();
    for line in existing.lines().skip(1) {
        if line.trim().is_empty() { continue; }
        let parts: Vec<&str> = line.splitn(2, ',').collect();
        if parts.len() >= 1 { seen.insert(parts[0].trim().trim_matches('"').to_string()); }
    }

    // Iterate knowledge/*.csv
    if let Ok(entries) = fs::read_dir(base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().map(|e| e == "csv").unwrap_or(false) {
                if let Ok(text) = fs::read_to_string(&path) {
                    for (i, line) in text.lines().enumerate() {
                        let l = line.trim();
                        if l.is_empty() || l.starts_with('#') { continue; }
                        // skip typical header lines
                        if i == 0 && (l.to_lowercase().contains("id") || l.to_lowercase().contains("topic") || l.to_lowercase().contains("question")) { continue; }
                        let parts: Vec<&str> = l.splitn(2, ',').collect();
                        if parts.len() < 2 { continue; }
                        let q = parts[0].trim().trim_matches('"').to_string();
                        let a = parts[1].trim().trim_matches('"').to_string();
                        if !seen.contains(&q) {
                            let mut f = OpenOptions::new().create(true).append(true).open(main_path)?;
                            writeln!(f, "\"{}\",\"{}\"", q.replace('"', "'"), a.replace('"', "'"))?;
                            seen.insert(q);
                        }
                    }
                }
            }
        }
    }

    // Log merge
    let mut log = OpenOptions::new().create(true).append(true).open("docs/knowledge_log.md")?;
    let now = Utc::now().to_rfc3339();
    writeln!(log, "\n### [{}] Объединение источников знаний", now)?;
    writeln!(log, "- объединены файлы из {}/ into crates/predict/data/knowledge.csv", base_dir)?;

    Ok(())
}
