use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_config, DatabaseSettings},
    startup::{get_pool, Application},
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

        let config = {
            let mut c = get_config().expect("Failed to read config");

            c.database.database_name = Uuid::new_v4().to_string();
            c.application.port = 0;
            c
        };

        // create and migrate database
        Self::configure_db(&config.database).await;

        let application = Application::build(config.clone())
            .await
            .expect("Failed to build application");

        // get port before spawing app
        let address = format!("http://127.0.0.1:{}", application.port());
        let _ = tokio::spawn(application.run_until_stopped());

        AppBootstrap {
            address,
            db_pool: get_pool(&config.database),
        }
    }

    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}
