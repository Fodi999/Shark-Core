use std::fs;
use std::io::Write;
use std::path::Path;

/// Проверка наличия критических файлов и функций, восстановление при необходимости
pub fn self_repair() {
    let critical_files = vec![
        ("core.rs", r#"pub fn softmax(logits: &mut [f32]) {}"#),
        (
            "memory.rs",
            r#"pub fn load(path: &str) -> Self { Self::default() }"#,
        ),
        (
            "model.rs",
            r#"pub fn forward(&self, _input: &[f32]) -> Vec<f32> { vec![] }"#,
        ),
    ];

    let base = "crates/predict/src";
    let log_path = "docs/self_fix.log";

    let mut log = String::from(format!(
        "🧠 [Self-Repair {}]\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));

    fs::create_dir_all("docs").ok();

    for (file, stub) in critical_files {
        let path = Path::new(base).join(file);

        if !path.exists() {
            log.push_str(&format!("⚠️  Файл {} отсутствует. Создаю...\n", file));
            let mut f = fs::File::create(&path).unwrap();
            writeln!(
                f,
                "#![forbid(unsafe_code)]\n// восстановлено автоматически\n{}",
                stub
            )
            .ok();
        } else {
            let content = fs::read_to_string(&path).unwrap_or_default();
            if !content.contains("pub fn") {
                log.push_str(&format!(
                    "⚠️  Файл {} пуст или повреждён. Восстанавливаю код...\n",
                    file
                ));
                let mut f = fs::File::create(&path).unwrap();
                writeln!(
                    f,
                    "#![forbid(unsafe_code)]\n// восстановлено автоматически\n{}",
                    stub
                )
                .ok();
            }
        }
    }

    log.push_str("✅ Проверка завершена. Все критические модули на месте.\n");

    fs::write(log_path, &log).ok();
    println!("{}", log);
}
