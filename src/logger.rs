use std::io::Write;

pub fn init() {
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{level} {time} {file}:{line}] - {message}",
                level = format!("{:<5}", record.level()),
                time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                file = record.file().unwrap_or("unknown"),
                line = record.line().unwrap_or(0),
                message = record.args(),
            )
        })
        .init();
}
