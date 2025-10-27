#![forbid(unsafe_code)]

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use rayon::prelude::*;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::time::Instant;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

// ---------- Законы (те же, что в тесте) ----------
fn law_wave(x: f64) -> f64 { 2.0 * (1.3 * x).sin() + 1.0 }
fn law_exp(x: f64) -> f64 { (0.3 * x).exp() - 1.0 }
fn law_power(x: f64) -> f64 { 0.5 * x.abs().powf(2.5) }
fn law_mixed(x: f64) -> f64 { 3.0 * (1.5 * x).sin() + 0.5 * x * x + 2.0 }
fn law_logistic(x: f64) -> f64 { 1.0 / (1.0 + (-x).exp()) }

fn xs_grid() -> Vec<f64> { (-100..=100).map(|i| i as f64 / 10.0).collect() }

// ---------- Символьная часть (минимально из e11_1) ----------
#[derive(Clone, Debug)]
enum Expr {
    Const(f64),
    X,
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Sin(Box<Expr>),
    Cos(Box<Expr>),
    Exp(Box<Expr>),
    Pow(Box<Expr>, f64),
    Scale(Box<Expr>, f64),
}
impl Expr {
    fn eval(&self, x: f64) -> f64 {
        match self {
            Expr::Const(c) => *c,
            Expr::X => x,
            Expr::Add(a,b) => a.eval(x) + b.eval(x),
            Expr::Mul(a,b) => a.eval(x) * b.eval(x),
            Expr::Sin(a) => a.eval(x).sin(),
            Expr::Cos(a) => a.eval(x).cos(),
            Expr::Exp(a) => a.eval(x).exp(),
            Expr::Pow(a,p) => {
                let base = a.eval(x);
                // защита от комплексных значений: если база не конечна — пометим как NaN
                if !base.is_finite() {
                    f64::NAN
                } else if base < 0.0 && p.fract() != 0.0 {
                    // отрицательные базы с дробными степенями → используем модуль базы
                    base.abs().powf(*p)
                } else {
                    base.powf(*p)
                }
            }
            Expr::Scale(a,k) => a.eval(x) * *k,
        }
    }
    fn node_count(&self) -> usize {
        match self {
            Expr::Const(_) | Expr::X | Expr::Pow(_,_) | Expr::Scale(_,_) => 1,
            Expr::Sin(a) | Expr::Cos(a) | Expr::Exp(a) => 1 + a.node_count(),
            Expr::Add(a,b) | Expr::Mul(a,b) => 1 + a.node_count() + b.node_count(),
        }
    }
}

fn rand_const(rng: &mut ChaCha8Rng) -> f64 { rng.gen_range(-3.0..3.0) }
fn rand_leaf(rng: &mut ChaCha8Rng) -> Expr {
    if rng.gen_bool(0.5) { Expr::X } else { Expr::Const(rand_const(rng)) }
}
fn rand_expr(rng: &mut ChaCha8Rng, depth: usize) -> Expr {
    if depth == 0 { return rand_leaf(rng); }
    match rng.gen_range(0..8) {
        0 => Expr::Add(Box::new(rand_expr(rng, depth-1)), Box::new(rand_expr(rng, depth-1))),
        1 => Expr::Mul(Box::new(rand_expr(rng, depth-1)), Box::new(rand_expr(rng, depth-1))),
        2 => Expr::Sin(Box::new(rand_expr(rng, depth-1))),
        3 => Expr::Cos(Box::new(rand_expr(rng, depth-1))),
        4 => Expr::Exp(Box::new(rand_expr(rng, depth-1))),
        5 => Expr::Pow(Box::new(rand_expr(rng, depth-1)), rng.gen_range(0.5..3.0)),
        6 => Expr::Scale(Box::new(rand_expr(rng, depth-1)), rng.gen_range(-3.0..3.0)),
        _ => rand_leaf(rng),
    }
}
fn mutate(e: &Expr, rng: &mut ChaCha8Rng, depth: usize) -> Expr {
    if rng.gen_bool(0.12) { return rand_expr(rng, depth.min(3)); }
    match e {
        Expr::Const(c) => Expr::Const(*c + rng.gen_range(-0.2..0.2)),
        Expr::X => Expr::X,
        Expr::Add(a,b) => Expr::Add(Box::new(mutate(a,rng,depth+1)), Box::new(mutate(b,rng,depth+1))),
        Expr::Mul(a,b) => Expr::Mul(Box::new(mutate(a,rng,depth+1)), Box::new(mutate(b,rng,depth+1))),
        Expr::Sin(a) => Expr::Sin(Box::new(mutate(a,rng,depth+1))),
        Expr::Cos(a) => Expr::Cos(Box::new(mutate(a,rng,depth+1))),
        Expr::Exp(a) => Expr::Exp(Box::new(mutate(a,rng,depth+1))),
        Expr::Pow(a,p) => Expr::Pow(Box::new(mutate(a,rng,depth+1)), p + rng.gen_range(-0.1..0.1)),
        Expr::Scale(a,k) => Expr::Scale(Box::new(mutate(a,rng,depth+1)), k + rng.gen_range(-0.2..0.2)),
    }
}
fn crossover(parent: &Expr, donor: &Expr, rng: &mut ChaCha8Rng, depth: usize) -> Expr {
    if rng.gen_bool(0.10) { return donor.clone(); }
    match parent {
        Expr::Const(_) | Expr::X | Expr::Pow(_,_) | Expr::Scale(_,_) => parent.clone(),
        Expr::Sin(a) => Expr::Sin(Box::new(crossover(a, donor, rng, depth+1))),
        Expr::Cos(a) => Expr::Cos(Box::new(crossover(a, donor, rng, depth+1))),
        Expr::Exp(a) => Expr::Exp(Box::new(crossover(a, donor, rng, depth+1))),
        Expr::Add(a,b) => {
            if rng.gen_bool(0.5) { Expr::Add(Box::new(crossover(a,donor,rng,depth+1)), b.clone()) }
            else { Expr::Add(a.clone(), Box::new(crossover(b,donor,rng,depth+1))) }
        }
        Expr::Mul(a,b) => {
            if rng.gen_bool(0.5) { Expr::Mul(Box::new(crossover(a,donor,rng,depth+1)), b.clone()) }
            else { Expr::Mul(a.clone(), Box::new(crossover(b,donor,rng,depth+1))) }
        }
    }
}

#[derive(Clone)]
struct LossCounter { limit: usize, used: Arc<AtomicUsize> }
impl LossCounter {
    fn new(limit: usize) -> Self { Self { limit, used: Arc::new(AtomicUsize::new(0)) } }
    fn tick(&self) { self.used.fetch_add(1, Ordering::SeqCst); }
    fn can(&self) -> bool { self.used.load(Ordering::SeqCst) < self.limit }
    fn used(&self) -> usize { self.used.load(Ordering::SeqCst) }
}

fn penalized_mse(expr: &Expr, y: &(dyn Fn(f64)->f64 + Sync), xs: &[f64], counter: &LossCounter) -> Option<f64> {
    if !counter.can() { return None; }
    // count this loss evaluation
    counter.tick();
    let sum: f64 = xs.par_iter().map(|&x| {
        let val = expr.eval(x);
        let d = if val.is_nan() || !val.is_finite() { 1e6 } else { val - y(x) };
        d * d
    }).sum::<f64>();
    let mse = sum / xs.len() as f64;
    let alpha = 0.01;
    Some(mse + alpha * (expr.node_count() as f64))
}
fn local_opt(expr: &Expr, rng: &mut ChaCha8Rng, steps: usize) -> Expr {
    let mut cur = expr.clone();
    for _ in 0..steps { if rng.gen_bool(0.5) { cur = mutate(&cur, rng, 0); } }
    cur
}

fn evolve_on(seed: u64, law: &(dyn Fn(f64)->f64 + Sync), xs: &[f64], eval_budget: usize, pop_size: usize) -> (Expr, f64, usize, u128) {
    let start = Instant::now();
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let counter = LossCounter::new(eval_budget);
    let mut pop: Vec<Expr> = (0..pop_size).map(|_| rand_expr(&mut rng, 3)).collect();
    let mut best = pop[0].clone();
    let mut best_fit = f64::INFINITY;

    while counter.can() {
        // evaluate population (shared counter via atomics)
        let fits: Vec<(usize,f64)> = pop.par_iter().enumerate()
            .filter_map(|(i,e)| penalized_mse(e, law, xs, &counter).map(|f|(i,f)))
            .collect();

        for (i,f) in &fits { if *f < best_fit { best_fit = *f; best = pop[*i].clone(); } }
        if !counter.can() { break; }

        let mut next = Vec::with_capacity(pop.len());
        next.push(best.clone());
        while next.len() < pop.len() && counter.can() {
            let i = rng.gen_range(0..pop.len());
            let j = rng.gen_range(0..pop.len());
            let child = crossover(&local_opt(&pop[i], &mut rng, 1), &pop[j], &mut rng, 0);
            let mut child = mutate(&child, &mut rng, 0);
            // reanimation: if trivial sanity check fails (NaN on few sample points), replace child
            let sample_ok = {
                let t0 = child.eval(0.0);
                let t1 = child.eval(1.0);
                t0.is_finite() && t1.is_finite()
            };
            if !sample_ok {
                child = rand_expr(&mut rng, 2);
            }
            next.push(child);
        }
        pop = next;
    }

    let elapsed_ms = start.elapsed().as_millis();
    let sum_clean: f64 = xs.par_iter().map(|&x| {
        let val = best.eval(x);
        let d = if val.is_nan() || !val.is_finite() { 1e6 } else { val - law(x) };
        d * d
    }).sum::<f64>();
    let mse_clean = sum_clean / xs.len() as f64;
    (best, mse_clean, counter.used(), elapsed_ms)
}

fn main() -> std::io::Result<()> {
    let xs = xs_grid();
    let eval_budget = 600_000usize; // увеличенный бюджет оценок
    let pop_size = 120usize; // увеличенный размер популяции
    let repeats = 7usize;          // несколько повторов для усреднения

    let tasks: Vec<(&str, fn(f64)->f64)> = vec![
        ("wave",     law_wave),
        ("exp",      law_exp),
        ("power",    law_power),
        ("mixed",    law_mixed),
        ("logistic", law_logistic),
    ];

    create_dir_all("docs")?;
    let mut csv = OpenOptions::new().create(true).append(true).open("docs/bench_e11_1.csv")?;
    writeln!(csv, "run,task,seed,evals_used,time_ms,mse")?;

    let mut total_mse = 0.0;
    let mut total_time = 0u128;
    let mut total_evals = 0usize;
    let mut n = 0usize;

    for r in 0..repeats {
        for (k, (name, law)) in tasks.iter().enumerate() {
            let seed = 1000 + r as u64 * 17 + k as u64;
            let (_expr, mse, used, ms) = evolve_on(seed, law, &xs, eval_budget, pop_size);
            println!("run #{:<2} | {:<8} | MSE={:.6} | evals={} | time_ms={}", r, name, mse, used, ms);
            writeln!(csv, "{},{},{},{},{},{}", r, name, seed, used, ms, mse)?;
            total_mse += mse; total_time += ms; total_evals += used; n += 1;
        }
    }

    let avg_mse = total_mse / n as f64;
    let avg_time_ms = total_time as f64 / n as f64;
    println!("\nRUST-BENCH | RUNS={} | AVG_MSE={:.6} | AVG_TIME_MS={:.1} | AVG_EVALS={:.0}", n, avg_mse, avg_time_ms, (total_evals as f64 / n as f64));
    Ok(())
}
