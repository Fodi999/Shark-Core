use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::fs;
use std::path::Path;

/// Простая экспериментальная подсистема "AI Scientist".
///
/// Содержит две части:
/// - `run_scientific_cycle` — простой гипотезогенератор и тестировщик (лог в `docs/`).
/// - `evolve_symbolic` — эволюционный поиск символьных формул (символьная регрессия).

/// Результат проверки гипотезы (для простого цикла).
#[derive(Debug)]
pub struct HypothesisResult {
    /// Имя или описание гипотезы
    pub name: String,
    /// Среднеквадратичная ошибка по выборке
    pub mse: f64,
    /// Флаг — гипотеза принята (true) или отклонена (false)
    pub accepted: bool,
}

/// Простая функция: измерение MSE против целевой функции
fn evaluate(f: fn(f64) -> f64, target: fn(f64) -> f64, n: usize) -> f64 {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let mut err = 0.0;
    for _ in 0..n {
        let x = rng.gen_range(-5.0..5.0);
        let diff = f(x) - target(x);
        err += diff * diff;
    }
    err / n as f64
}

/// Запуск простого научного цикла: формула -> оценка -> лог.
pub fn run_scientific_cycle() -> Vec<HypothesisResult> {
    // Истинная функция (можно заменить на более сложную/шумную в будущем)
    fn true_fn(x: f64) -> f64 { 0.5 * x * x + 1.0 * x + 2.0 }

    let hypotheses: Vec<(&str, fn(f64)->f64)> = vec![
        ("Quadratic model", |x| 0.4 * x * x + 1.0 * x + 2.1),
        ("Hybrid sin+quad", |x| 0.5 * x.sin() + 0.5 * x * x + 2.0),
        ("Cosine-modulated", |x| (x * 1.1).cos() + 0.4 * x + 1.8),
    ];

    let mut results = Vec::new();
    for (name, f) in hypotheses {
        let mse = evaluate(f, true_fn, 10_000);
        // Acceptance threshold: tuned for toy scenario
        let accepted = mse < 1.2;
        results.push(HypothesisResult { name: name.to_string(), mse, accepted });
    }

    // Логируем отчёт в docs/AI_SCIENTIST_REPORT.md, но не паниковать при ошибке записи
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("docs/AI_SCIENTIST_REPORT.md") {
        let now = chrono::Utc::now().to_rfc3339();
        let _ = writeln!(file, "### Эксперимент от {now}\n");
        for r in &results {
            let _ = writeln!(file, "- {} → MSE={:.4} {}", r.name, r.mse, if r.accepted { "✅" } else { "❌" });
        }
    } else {
        // fallback: печатаем в stdout
        println!("AI Scientist: не удалось открыть docs/AI_SCIENTIST_REPORT.md для записи");
        for r in &results {
            println!("- {} → MSE={:.4} {}", r.name, r.mse, if r.accepted { "✅" } else { "❌" });
        }
    }

    results
}

/// Символьное выражение (простое DSL для эволюционного поиска).
///
/// Представляет небольшое дерево выражения, которое можно оценить на x.
#[derive(Clone)]
pub enum Expr {
    /// Константа
    Const(f64),
    /// Переменная x
    X,
    /// Сложение
    Add(Box<Expr>, Box<Expr>),
    /// Умножение
    Mul(Box<Expr>, Box<Expr>),
    /// sin(...)
    Sin(Box<Expr>),
    /// cos(...)
    Cos(Box<Expr>),
    /// Квадрат выражения
    Pow2(Box<Expr>),
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Const(c) => write!(f, "{:.3}", c),
            Expr::X => write!(f, "x"),
            Expr::Add(a,b) => write!(f, "({:?}+{:?})", a,b),
            Expr::Mul(a,b) => write!(f, "({:?}*{:?})", a,b),
            Expr::Sin(a) => write!(f, "sin({:?})", a),
            Expr::Cos(a) => write!(f, "cos({:?})", a),
            Expr::Pow2(a) => write!(f, "({:?})^2", a),
        }
    }
}

impl Expr {
    /// Вычислить значение выражения при данном x.
    pub fn eval(&self, x: f64) -> f64 {
        match self {
            Expr::Const(c) => *c,
            Expr::X => x,
            Expr::Add(a,b) => a.eval(x) + b.eval(x),
            Expr::Mul(a,b) => a.eval(x) * b.eval(x),
            Expr::Sin(a) => a.eval(x).sin(),
            Expr::Cos(a) => a.eval(x).cos(),
            Expr::Pow2(a) => {
                let v = a.eval(x);
                v * v
            }
        }
    }
}

fn rand_const(rng: &mut ChaCha8Rng) -> f64 {
    // компактный диапазон для стабильности
    rng.gen_range(-3.0..3.0)
}

fn rand_leaf(rng: &mut ChaCha8Rng) -> Expr {
    if rng.gen_bool(0.5) { Expr::X } else { Expr::Const(rand_const(rng)) }
}

fn rand_expr(rng: &mut ChaCha8Rng, depth: usize) -> Expr {
    if depth == 0 {
        return rand_leaf(rng);
    }
    match rng.gen_range(0..6) {
        0 => Expr::Add(Box::new(rand_expr(rng, depth-1)), Box::new(rand_expr(rng, depth-1))),
        1 => Expr::Mul(Box::new(rand_expr(rng, depth-1)), Box::new(rand_expr(rng, depth-1))),
        2 => Expr::Sin(Box::new(rand_expr(rng, depth-1))),
        3 => Expr::Cos(Box::new(rand_expr(rng, depth-1))),
        4 => Expr::Pow2(Box::new(rand_expr(rng, depth-1))),
        _ => rand_leaf(rng),
    }
}

fn mutate(expr: &Expr, rng: &mut ChaCha8Rng, depth: usize) -> Expr {
    // с малой вероятностью — заменить поддерево
    if rng.gen_bool(0.15) {
        return rand_expr(rng, depth.min(3));
    }
    match expr {
        Expr::Const(_) if rng.gen_bool(0.6) => Expr::Const(rand_const(rng)),
        Expr::Const(c) => Expr::Const(*c + rng.gen_range(-0.2..0.2)),
        Expr::X => {
            if rng.gen_bool(0.1) { Expr::Const(rand_const(rng)) } else { Expr::X }
        }
        Expr::Add(a,b) => Expr::Add(Box::new(mutate(a, rng, depth+1)), Box::new(mutate(b, rng, depth+1))),
        Expr::Mul(a,b) => Expr::Mul(Box::new(mutate(a, rng, depth+1)), Box::new(mutate(b, rng, depth+1))),
        Expr::Sin(a) => {
            if rng.gen_bool(0.1) { Expr::Cos(a.clone()) } else { Expr::Sin(Box::new(mutate(a, rng, depth+1))) }
        }
        Expr::Cos(a) => {
            if rng.gen_bool(0.1) { Expr::Sin(a.clone()) } else { Expr::Cos(Box::new(mutate(a, rng, depth+1))) }
        }
        Expr::Pow2(a) => Expr::Pow2(Box::new(mutate(a, rng, depth+1))),
    }
}

/// MSE на выборке из N точек
fn mse(expr: &Expr, target: fn(f64)->f64, rng: &mut ChaCha8Rng, n: usize) -> f64 {
    let mut s = 0.0;
    for _ in 0..n {
        let x = rng.gen_range(-5.0..5.0);
        let y = target(x);
        let yhat = expr.eval(x);
        let d = yhat - y;
        s += d*d;
    }
    s / n as f64
}

/// Турнирный отбор — возвращает ссылку на выбранного члена популяции
fn tournament<'a>(pop: &'a [Expr], fits: &[f64], rng: &mut ChaCha8Rng, k: usize) -> &'a Expr {
    // Инициализируем с случайного кандидата, затем проводим k-1 состязаний
    let mut best_idx = rng.gen_range(0..pop.len());
    let mut best_fit = fits[best_idx];
    for _ in 1..k {
        let i = rng.gen_range(0..pop.len());
        let f = fits[i];
        if f < best_fit {
            best_fit = f;
            best_idx = i;
        }
    }
    &pop[best_idx]
}

/// Эволюционный поиск формулы.
///
/// Возвращает лучшую найденную формулу и её MSE на большой выборке.
pub fn evolve_symbolic(seed: u64, generations: usize, pop_size: usize) -> (Expr, f64) {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    // неизвестная "истинная" функция (сложно-нелинейная)
    fn target(x: f64) -> f64 { (1.2*x).sin() + 0.4*x*x + 0.8*x + 0.5 }

    // инициализация
    let mut pop: Vec<Expr> = (0..pop_size).map(|_| rand_expr(&mut rng, 3)).collect();

    // основной цикл
    let mut best_expr = pop[0].clone();
    let mut best_fit = f64::INFINITY;

    for gen in 0..generations {
        // фитнесы
        let mut fits: Vec<f64> = Vec::with_capacity(pop.len());
        for e in &pop {
            let f = mse(e, target, &mut rng, 200); // быстрая оценка
            fits.push(f);
            if f < best_fit {
                best_fit = f;
                best_expr = e.clone();
            }
        }

        // эволюция: селекция + мутации
        let mut next = Vec::with_capacity(pop_size);
        next.push(best_expr.clone()); // элитизм
        while next.len() < pop_size {
            let p = tournament(&pop, &fits, &mut rng, 3).clone();
            let c = mutate(&p, &mut rng, 0);
            next.push(c);
        }
        pop = next;

        if gen % 50 == 0 {
            // периодическая перепроверка на большой выборке
            let check_fit = mse(&best_expr, target, &mut rng, 2000);
            // лёгкий лог в консоль
            println!("gen {gen:>4}: best MSE ~ {:.4}", check_fit);
        }
    }

    // финальная оценка на большой выборке
    let final_fit = mse(&best_expr, target, &mut rng, 5000);

    // Сохранить открытие в память ученого
    let name = format!("evolve_{}_{:x}", seed, chrono::Utc::now().timestamp());
    let formula = format!("{:?}", best_expr);
    let _ = save_discovery(&name, &formula, final_fit);
    // also write the simpler CSV record (formula,mse,curiosity,date)
    log_discovery(&best_expr, final_fit).ok();

    (best_expr, final_fit)
}

/// Вычислить критерий любознательности (curiosity) из MSE
pub fn curiosity_from_mse(mse: f64) -> f64 {
    1.0 / (1.0 + mse)
}

/// Сохранить открытие (имя, формула, mse) в `crates/predict/data/knowledge_science.csv`.
pub fn save_discovery(name: &str, formula: &str, mse: f64) -> std::io::Result<()> {
    let dir = Path::new("crates/predict/data");
    fs::create_dir_all(dir)?;
    let file_path = dir.join("knowledge_science.csv");

    let header = "name,formula,mse,curiosity\n";
    let exists = file_path.exists();
    if !exists {
        // create with header
        fs::write(&file_path, header)?;
    }

    let curiosity = curiosity_from_mse(mse);
    let mut f = OpenOptions::new().create(true).append(true).open(&file_path)?;
    // quote fields to be safe
    let row = format!("\"{}\",\"{}\",{:.6},{:.6}\n", name.replace('"', "'"), formula.replace('"', "'"), mse, curiosity);
    f.write_all(row.as_bytes())?;
    Ok(())
}

/// Лёгкая запись открытия: формула, mse, curiosity, дата
pub fn log_discovery(expr: &Expr, mse: f64) -> std::io::Result<()> {
    let dir = Path::new("crates/predict/data");
    fs::create_dir_all(dir)?;
    let file_path = dir.join("knowledge_science.csv");

    let header = "formula,mse,curiosity,date\n";
    if !file_path.exists() {
        fs::write(&file_path, header)?;
    }

    let curiosity = curiosity_from_mse(mse);
    let now = chrono::Utc::now().to_rfc3339();
    let mut f = OpenOptions::new().create(true).append(true).open(&file_path)?;
    let row = format!("\"{:?}\",{:.6},{:.6},\"{}\"\n", expr, mse, curiosity, now);
    f.write_all(row.as_bytes())?;
    Ok(())
}

/// Простая загрузка памяти открытий (name,formula,mse,curiosity)
pub fn load_science_memory() -> Vec<(String, String, f64, f64)> {
    let file_path = Path::new("crates/predict/data/knowledge_science.csv");
    let mut out = Vec::new();
    if !file_path.exists() {
        return out;
    }
    if let Ok(s) = fs::read_to_string(file_path) {
        for (i, line) in s.lines().enumerate() {
            if i == 0 { continue; } // skip header
            // naive CSV split: name,formula,mse,curiosity
            let parts: Vec<&str> = line.splitn(4, ',').collect();
            if parts.len() < 4 { continue; }
            let name = parts[0].trim().trim_matches('"').to_string();
            let formula = parts[1].trim().trim_matches('"').to_string();
            let mse = parts[2].trim().parse::<f64>().unwrap_or(f64::INFINITY);
            let curiosity = parts[3].trim().parse::<f64>().unwrap_or(0.0);
            out.push((name, formula, mse, curiosity));
        }
    }
    out
}
 
