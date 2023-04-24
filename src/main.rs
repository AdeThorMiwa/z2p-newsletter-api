use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;
use zero2prod::{
    configuration::get_config,
    services::email::EmailService,
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
        .connect_lazy_with(config.database.with_db());

    let port = std::env::var("PORT").unwrap_or(format!("{}", config.application.port));
    let url = format!("{}:{}", config.application.host, port);
    let listener = TcpListener::bind(url)?;

    let email_sender = config
        .email
        .sender()
        .expect("Invalid sender email address.");
    let email_service =
        EmailService::new(config.email.base_url, email_sender, config.email.auth_token);

    run(listener, pool, email_service)?.await
}
