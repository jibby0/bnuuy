use std::io;

pub fn setup_logging(verbosity: log::LevelFilter) -> Result<(), fern::InitError> {
    let base_config = fern::Dispatch::new().level(verbosity);

    let stdout_config = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .chain(io::stdout());

    base_config.chain(stdout_config).apply()?;

    Ok(())
}
