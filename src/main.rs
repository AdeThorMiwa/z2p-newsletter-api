use secrecy::ExposeSecret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use zero2prod::{
    configuration::get_config,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Setup logging
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we cant read config
    let config = get_config().expect("Failed to read configuration.");
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&config.database.connection_string().expose_secret())
        .expect("Failed to connect to Postgres");

    let url = format!("{}:{}", config.application.host, config.application.port);
    let listener = TcpListener::bind(url)?;
    run(listener, pool)?.await
}
