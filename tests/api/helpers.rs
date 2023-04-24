use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    configuration::{get_config, DatabaseSettings},
    services::email::EmailService,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct AppBootstrap {
    pub address: String,
    pub db_pool: PgPool,
}

impl AppBootstrap {
    async fn configure_db(config: &DatabaseSettings) -> PgPool {
        // create db
        let mut conn = PgConnection::connect_with(&config.without_db())
            .await
            .expect("Failed to connect to Postgres");

        conn.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
            .await
            .expect("Failed to create db");

        let conn_pool = PgPool::connect_with(config.with_db())
            .await
            .expect("Failed to connect to Postgres.");

        sqlx::migrate!("./migrations")
            .run(&conn_pool)
            .await
            .expect("Failed to migrate the db");

        conn_pool
    }

    pub async fn new() -> Self {
        Lazy::force(&TRACING);

        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        let mut config = get_config().expect("Failed to read config");
        config.database.database_name = Uuid::new_v4().to_string();
        let db_pool = Self::configure_db(&config.database).await;

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

        let server = run(listener, db_pool.clone(), email_service).expect("Failed to bind address");

        let _ = tokio::spawn(server);

        AppBootstrap { address, db_pool }
    }
}
