use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

pub fn init_logging() {
    Builder::new()
        .filter(None, LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}
