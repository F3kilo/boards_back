pub mod boards;
mod db;
mod errors;
mod handlers;
mod models;
pub mod rate_lim;
mod tasks;

use crate::boards::Boards;
use crate::db::cached::Cached;
use crate::db::mongo::Mongo;
use crate::tasks::Tasks;
use actix_web::{web, App, HttpServer};
use std::env;
use std::sync::Arc;
use crate::rate_lim::RateLimiter;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    init()?;

    let mongo_connection_str = env::var("MONGO_CONNECTION")?;
    let client = mongodb::Client::with_uri_str(mongo_connection_str).await?;
    let mongo_db = Mongo::new(client);

    let redis_connection_str = env::var("REDIS_CONNECTION")?;
    let redis_client = redis::Client::open(redis_connection_str)?;
    let connection_manager = redis_client.get_tokio_connection_manager().await?;
    let rate_limiter = RateLimiter::new(connection_manager);
    let database = Box::new(Cached::new(mongo_db, redis_client));



    let boards = Arc::new(Boards::new(database.clone()));
    let tasks = Arc::new(Tasks::new(database));


    HttpServer::new(move || {
        App::new()
            // boards
            .service(handlers::read_boards)
            .service(handlers::create_board)
            .service(handlers::read_board)
            .service(handlers::update_board)
            .service(handlers::delete_board)
            .service(handlers::subscribe_board_changes)
            // tasks
            .service(handlers::read_tasks)
            .service(handlers::read_tasks)
            .service(handlers::create_task)
            .service(handlers::read_task)
            .service(handlers::update_task)
            .service(handlers::delete_task)
            // config
            .wrap(actix_web::middleware::Logger::default())
            .wrap(rate_limiter.clone())
            .app_data(web::Data::new(Arc::clone(&boards)))
            .app_data(web::Data::new(Arc::clone(&tasks)))
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
