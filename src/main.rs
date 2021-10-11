mod boards;
mod rate_lim;

use crate::boards::Boards;
use crate::rate_lim::RateLimiter;
use actix_web::{App, HttpServer};
use std::env;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    init()?;

    let boards = Arc::new(Boards::new());
    let rate_limiter = Arc::new(RateLimiter::new());

    HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(Arc::clone(&boards))
            .app_data(Arc::clone(&rate_limiter))
    })
    .bind("127.0.0.1:9000")?
    .run()
    .await?;

    Ok(())
}

fn init() -> Result<(), fern::InitError> {
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "INFO".into());
    let log_level = log_level.parse().unwrap_or(log::LevelFilter::Info);

    let mut builder = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(std::io::stderr());

    if let Ok(log_file) = env::var("LOG_FILE") {
        let log_file = std::fs::File::create(log_file)?;
        builder = builder.chain(log_file);
    }

    builder.apply()?;

    log::trace!("TRACE output enabled");
    log::debug!("DEBUG output enabled");
    log::info!("INFO output enabled");
    log::warn!("WARN output enabled");
    log::error!("ERROR output enabled");

    Ok(())
}
