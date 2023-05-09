use crate::{
    configuration::{DatabaseSettings, Settings},
    routes::{confirm, health_check, home, login, login_form, post_newsletter, subscribe},
    services::email::EmailService,
};
use actix_web::{dev::Server, web, App, HttpServer};
use secrecy::Secret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);

#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

impl Application {
    pub async fn build(config: Settings) -> Result<Self, std::io::Error> {
        let pool = get_pool(&config.database);

        let port = std::env::var("PORT").unwrap_or(format!("{}", config.application.port));
        let url = format!("{}:{}", config.application.host, port);
        let listener = TcpListener::bind(url)?;

        let email_sender = config
            .email
            .sender()
            .expect("Invalid sender email address.");
        let timeout = config.email.timeout();
        let email_service = EmailService::new(
            config.email.base_url,
            email_sender,
            config.email.auth_token,
            timeout,
        );

        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            pool,
            email_service,
            config.application.base_url,
            config.application.hmac_secret,
        )?;

        Ok(Self { server, port })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_service: EmailService,
    base_url: String,
    hmac_secret: Secret<String>,
) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_service = web::Data::new(email_service);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let hmac_secret = web::Data::new(HmacSecret(hmac_secret.clone()));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .route("/newsletters", web::post().to(post_newsletter))
            .route("/", web::get().to(home))
            .route("/login", web::get().to(login_form))
            .route("/login", web::post().to(login))
            .app_data(db_pool.clone())
            .app_data(email_service.clone())
            .app_data(base_url.clone())
            .app_data(hmac_secret.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

pub fn get_pool(db_config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(db_config.with_db())
}
