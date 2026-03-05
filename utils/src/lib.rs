use std::time::Instant;

pub fn time_ms<F, R>(label: &str, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    log::info!("{} finished in {:.2} ms", label, elapsed_ms);
    result
}
