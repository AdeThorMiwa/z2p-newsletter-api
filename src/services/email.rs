use reqwest::Client;

use crate::domain::subscriber_email::SubscriberEmail;

pub struct EmailService {
    client: Client,
    base_url: String,
    sender: SubscriberEmail,
}

impl EmailService {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            client: Client::new(),
            base_url,
            sender,
        }
    }

    pub fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}
