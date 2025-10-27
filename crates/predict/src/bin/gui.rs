use eframe::{egui, App, Frame};
use predict::{AI, scientist};
use std::sync::{Arc, Mutex};
use std::thread;
use std::fs;
use std::time::{Instant, Duration};

#[derive(Clone, PartialEq)]
enum Tab {
    Chat,
    Research,
    Memory,
    Settings,
    Metrics,
}

struct SharkApp {
    ai: Arc<Mutex<AI>>,
    input: String,
    output: String,
    history: Vec<(String, String)>,
    tab: Tab,
    science_results: Vec<String>,
    memory_text: String,
    memory_rows: Vec<Vec<String>>,
    scientist_running: bool,
    scientist_output: Option<Arc<Mutex<Vec<String>>>>,
    // settings
    model_path: String,
    response_speed: f32,
    dark_mode: bool,
    enable_semantic: bool,
    auto_save_history: bool,
    // new fields
    progress: f32,
    progress_start: Option<Instant>,
    thinking: bool,
    training: bool,
    pending_reply: Option<Arc<Mutex<Option<(String, bool)>>>>,
    last_prompt: String,
    // metrics
    question_count: usize,
    total_response_time: f64,
    semantic_responses: usize,
    model_responses: usize,
    // history with timestamps
    history_with_time: Vec<(String, String, String)>, // (time, question, answer)
    start_time: Option<Instant>,
}

impl Default for SharkApp {
    fn default() -> Self {
        Self {
            ai: Arc::new(Mutex::new(AI::new("weights/model_int4.bin"))),
            input: String::new(),
            output: "🦈 Shark-Core готов к работе.".to_string(),
            history: Vec::new(),
            tab: Tab::Chat,
            science_results: Vec::new(),
            memory_text: String::new(),
            memory_rows: Vec::new(),
            scientist_running: false,
            scientist_output: None,
            model_path: "weights/model_int4.bin".to_string(),
            response_speed: 1.0,
            dark_mode: true,
            enable_semantic: true,
            auto_save_history: false,
            progress: 0.0,
            progress_start: None,
            thinking: false,
            training: false,
            pending_reply: None,
            last_prompt: String::new(),
            // metrics
            question_count: 0,
            total_response_time: 0.0,
            semantic_responses: 0,
            model_responses: 0,
            history_with_time: Vec::new(),
            start_time: None,
        }
    }
}

impl SharkApp {
    fn send_prompt(&mut self, ctx: &egui::Context) {
        let prompt = self.input.trim().to_owned();
        if prompt.is_empty() {
            return;
        }

        self.thinking = true;
        self.output = "🧠 думает...".to_string();
        self.last_prompt = prompt.clone();
        self.start_time = Some(Instant::now());

        // prepare shared slot for reply
        let reply_slot: Arc<Mutex<Option<(String, bool)>>> = Arc::new(Mutex::new(None));
        self.pending_reply = Some(reply_slot.clone());

        // clone Arc to move into thread
        let ai_arc = self.ai.clone();
        let prompt_clone = prompt.clone();
        let enable_semantic = self.enable_semantic;
        let thread_ctx = ctx.clone();
        thread::spawn(move || {
            // call model under lock
            let (reply_raw, is_semantic) = {
                let mut ai = ai_arc.lock().unwrap();
                if enable_semantic {
                    if let Some(semantic_reply) = predict::interpret_question(&prompt_clone, &ai.knowledge) {
                        (semantic_reply, true)
                    } else {
                        (ai.chat(&prompt_clone), false)
                    }
                } else {
                    (ai.chat(&prompt_clone), false)
                }
            };
            // store reply and type
            if let Ok(mut g) = reply_slot.lock() {
                *g = Some((reply_raw, is_semantic));
            }
            // request UI repaint
            thread_ctx.request_repaint();
        });

        // clear input while thinking
        self.input.clear();
    }

    fn finish_prompt(&mut self, reply_raw: String, is_semantic: bool) {
        // calculate response time
        let response_time = if let Some(start) = self.start_time.take() {
            start.elapsed().as_secs_f64()
        } else {
            0.0
        };

        // avoid destructive filtering: remove control characters but keep punctuation
        let mut cleaned: String = reply_raw.chars().filter(|c| !c.is_control()).collect();
        // trim extraneous whitespace at ends
        cleaned = cleaned.trim().to_string();

        // update metrics
        self.question_count += 1;
        self.total_response_time += response_time;
        if is_semantic {
            self.semantic_responses += 1;
        } else {
            self.model_responses += 1;
        }

        // add to history with time
        let now = chrono::Utc::now().format("%H:%M:%S").to_string();
        self.history_with_time.push((now, self.last_prompt.clone(), cleaned.clone()));

        self.history.push((self.last_prompt.clone(), cleaned.clone()));
        self.output = cleaned;
        self.thinking = false;
        self.pending_reply = None;
    }

    fn load_memory(&mut self) {
        // read knowledge.csv relative to the crate manifest dir
        let path = format!("{}/data/knowledge.csv", env!("CARGO_MANIFEST_DIR"));
        match fs::read_to_string(&path) {
            Ok(s) => self.memory_text = s,
            Err(e) => self.memory_text = format!("Failed to read {}: {}", path, e),
        }
        // also parse CSV into rows (naive split)
        self.memory_rows.clear();
        if !self.memory_text.is_empty() {
            for (i, line) in self.memory_text.lines().enumerate() {
                if i == 0 { continue; } // skip header if present
                let cols: Vec<String> = line.split(',').map(|c| c.trim().trim_matches('"').to_string()).collect();
                if !cols.is_empty() {
                    self.memory_rows.push(cols);
                }
            }
        }
    }
}

impl App for SharkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // check pending reply from background thread
        if let Some(slot) = self.pending_reply.as_ref().map(|s| s.clone()) {
            if let Ok(mut guard) = slot.lock() {
                if let Some((reply, is_semantic)) = guard.take() {
                    // process reply on UI thread
                    self.finish_prompt(reply, is_semantic);
                }
            }
        }
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🧠 Shark-Core");
                ui.separator();
                if ui.selectable_label(self.tab == Tab::Chat, "🧠 Чат").clicked() {
                    self.tab = Tab::Chat;
                }
                if ui.selectable_label(self.tab == Tab::Research, "🔬 Исследования").clicked() {
                    self.tab = Tab::Research;
                }
                if ui.selectable_label(self.tab == Tab::Memory, "📚 Память").clicked() {
                    self.tab = Tab::Memory;
                    self.load_memory();
                }
                if ui.selectable_label(self.tab == Tab::Settings, "⚙️ Настройки").clicked() {
                    self.tab = Tab::Settings;
                }
                if ui.selectable_label(self.tab == Tab::Metrics, "📊 Метрики").clicked() {
                    self.tab = Tab::Metrics;
                }
            });
        });

        // Bottom panel for input (only in Chat tab)
        if self.tab == Tab::Chat {
            egui::TopBottomPanel::bottom("input_panel").show(ctx, |ui| {
                ui.add_space(4.0);
                if self.thinking {
                    ui.colored_label(egui::Color32::LIGHT_BLUE, "🧠 Думает...");
                    ctx.request_repaint_after(Duration::from_millis(150));
                }
                ui.horizontal(|ui| {
                    ui.label("Вопрос:");
                    let response = ui.text_edit_singleline(&mut self.input);
                    if ui.button("Отправить").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        self.send_prompt(ctx);
                    }
                });
                ui.label("Текущий ответ:");
                ui.add(egui::TextEdit::multiline(&mut self.output).desired_rows(2));
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.tab {
                Tab::Chat => {
                    ui.vertical_centered(|ui| {
                        ui.label("Переписка:");
                        ui.horizontal(|ui| {
                            ui.add_space(100.0); // Left margin
                            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                                for (time, question, answer) in &self.history_with_time {
                                    // User message (right aligned)
                                    ui.horizontal(|ui| {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                            ui.small(format!("{}", time));
                                            ui.colored_label(egui::Color32::from_rgb(100, 150, 255), format!("Вы: {}", question));
                                        });
                                    });
                                    ui.add_space(4.0);
                                    // AI message (left aligned)
                                    ui.horizontal(|ui| {
                                        ui.small(format!("{}", time));
                                        ui.colored_label(egui::Color32::from_rgb(150, 150, 150), format!("Shark-Core: {}", answer));
                                    });
                                    ui.add_space(8.0);
                                }
                            });
                            ui.add_space(100.0); // Right margin
                        });
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        if ui.button("Очистить историю").clicked() {
                            self.history.clear();
                            self.history_with_time.clear();
                        }
                        if ui.button("Сохранить историю").clicked() {
                            let content = self.history_with_time.iter()
                                .map(|(t, q, a)| format!("{} | {} | {}", t, q, a))
                                .collect::<Vec<_>>()
                                .join("\n");
                            let path = "history.txt";
                            if let Err(e) = std::fs::write(path, content) {
                                self.output = format!("Ошибка сохранения: {}", e);
                            } else {
                                self.output = format!("История сохранена в {}", path);
                            }
                        }
                    });
                }

                Tab::Research => {
                    ui.horizontal(|ui| {
                        if ui.button("Исследовать").clicked() && !self.scientist_running {
                            self.scientist_running = true;
                            self.progress = 0.0;
                            self.progress_start = Some(Instant::now());
                            let results_arc: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
                            let thread_arc = results_arc.clone();
                            // save Arc so UI can poll it
                            self.scientist_output = Some(results_arc.clone());
                            // spawn background thread to run the scientist cycle
                            thread::spawn(move || {
                                let res = scientist::run_scientific_cycle();
                                let mut guard = thread_arc.lock().unwrap();
                                for r in res {
                                    guard.push(format!("{} | mse={:.6} | accepted={}", r.name, r.mse, r.accepted));
                                }
                            });
                            self.science_results.clear();
                            self.science_results.push("Запущено: исследовательский цикл...".to_string());
                        }
                        if ui.button("Обновить результаты").clicked() {
                            // try to load saved science memory or just show a message
                            // For now, read docs/knowledge_log.md as a proxy
                            let path = format!("{}/../docs/knowledge_log.md", env!("CARGO_MANIFEST_DIR"));
                            match fs::read_to_string(&path) {
                                Ok(s) => {
                                    self.science_results = s.lines().take(200).map(|l| l.to_string()).collect();
                                }
                                Err(e) => self.science_results = vec![format!("failed to read {}: {}", path, e)],
                            }
                        }
                    });

                    // Poll background results (if any)
                    if let Some(arc_clone) = self.scientist_output.as_ref().map(|a| a.clone()) {
                        if let Ok(mut g) = arc_clone.lock() {
                            if !g.is_empty() {
                                let copied = g.clone();
                                g.clear();
                                drop(g);
                                self.science_results = copied;
                                self.scientist_running = false;
                                // remove the stored arc so we don't poll again
                                self.scientist_output = None;
                            }
                        }
                    }

                    // Прогресс-бар исследования
                    if self.scientist_running {
                        if let Some(start) = self.progress_start {
                            let elapsed = start.elapsed().as_secs_f32();
                            self.progress = (elapsed / 10.0).min(1.0); // псевдо-прогресс на 10 секунд
                            ui.add(egui::ProgressBar::new(self.progress).text("🔬 Идёт исследование..."));
                        }
                    } else if self.progress >= 1.0 {
                        ui.label("✅ Исследование завершено!");
                    }

                    ui.separator();
                    ui.label("Результаты исследований:");
                    egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                        egui::Grid::new("science_grid").striped(true).show(ui, |ui| {
                            ui.label("Name"); ui.label("MSE"); ui.label("Accepted"); ui.end_row();
                            for line in &self.science_results {
                                // lines are formatted as: "<name> | mse=<val> | accepted=<bool>"
                                let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
                                let name = parts.get(0).copied().unwrap_or("");
                                let mse_part = parts.iter().find(|p| p.starts_with("mse=")).copied().unwrap_or("");
                                let accepted_part = parts.iter().find(|p| p.starts_with("accepted=")).copied().unwrap_or("");
                                ui.label(name);
                                ui.label(mse_part.replace("mse=", ""));
                                ui.label(accepted_part.replace("accepted=", ""));
                                ui.end_row();
                            }
                        });
                    });
                }

                Tab::Memory => {
                    ui.horizontal(|ui| {
                        if ui.button("Обновить память").clicked() {
                            self.load_memory();
                        }
                        ui.label(" ");
                        if ui.button("Показать как текст").clicked() {
                            // noop: memory_text already set
                        }
                    });

                    ui.separator();
                    // show table header and rows
                    egui::ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                        if !self.memory_rows.is_empty() {
                            // header from first row length or use default columns
                            egui::Grid::new("memory_grid").striped(true).show(ui, |ui| {
                                // show up to 6 columns for readability
                                for r in &self.memory_rows {
                                    for cell in r.iter().take(6) {
                                        ui.label(cell);
                                    }
                                    ui.end_row();
                                }
                            });
                        } else {
                            ui.add(egui::TextEdit::multiline(&mut self.memory_text).desired_rows(20).interactive(false));
                        }
                    });
                }

                Tab::Settings => {
                    ui.label("Настройки");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Model path:");
                        if ui.text_edit_singleline(&mut self.model_path).lost_focus() {
                            // reload model if changed
                            if let Ok(mut ai_lock) = self.ai.lock() {
                                *ai_lock = AI::new(&self.model_path);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Response speed:");
                        ui.add(egui::Slider::new(&mut self.response_speed, 0.1..=5.0).text("x"));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        if ui.selectable_label(self.dark_mode, "Тёмная").clicked() {
                            self.dark_mode = true;
                            ctx.set_visuals(egui::Visuals::dark());
                        }
                        if ui.selectable_label(!self.dark_mode, "Светлая").clicked() {
                            self.dark_mode = false;
                            ctx.set_visuals(egui::Visuals::light());
                        }
                    });
                    ui.separator();
                    ui.checkbox(&mut self.enable_semantic, "Включить семантическое понимание");
                    ui.checkbox(&mut self.auto_save_history, "Автосохранение истории");
                }

                Tab::Metrics => {
                    ui.label("📊 Метрики Shark-Core");
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Всего вопросов:");
                        ui.label(self.question_count.to_string());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Среднее время ответа:");
                        let avg_time = if self.question_count > 0 {
                            self.total_response_time / self.question_count as f64
                        } else {
                            0.0
                        };
                        ui.label(format!("{:.2} сек", avg_time));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Семантических ответов:");
                        ui.label(self.semantic_responses.to_string());
                    });
                    ui.horizontal(|ui| {
                        ui.label("Ответов модели:");
                        ui.label(self.model_responses.to_string());
                    });
                    ui.separator();
                    if ui.button("Сбросить метрики").clicked() {
                        self.question_count = 0;
                        self.total_response_time = 0.0;
                        self.semantic_responses = 0;
                        self.model_responses = 0;
                    }
                }
            }
        });
    }

}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Shark-Core GUI 🦈",
        native_options,
        Box::new(|_| Ok(Box::new(SharkApp::default()))),
    )
}
 
