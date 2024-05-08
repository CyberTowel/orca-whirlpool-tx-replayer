#[macro_export]
macro_rules! dbgtest {
    ($expr:expr) => {{
        let start = std::time::Instant::now();
        let value = $expr;
        let duration = start.elapsed();
        eprintln!(
            "[{}:{}] {} = {:#?} ({:?})",
            file!(),
            line!(),
            stringify!($expr),
            value,
            duration
        );
        value
    }};
}
