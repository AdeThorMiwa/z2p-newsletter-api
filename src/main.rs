use zero2prod::{
    configuration::get_config,
    startup::Application,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Setup logging
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we cant read config
    let config = get_config().expect("Failed to read configuration.");
    let application = Application::build(config).await?;
    application.run_until_stopped().await?;
    Ok(())
}
