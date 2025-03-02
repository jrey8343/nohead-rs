use nohead_rs_config::Config;

use crate::{EmailClient, EmailPayload, Error};

pub struct AuthMailer;

impl AuthMailer {
    pub async fn send_confirmation(
        config: &Config,
        email_client: &EmailClient,
        email_recipient: &str,
        register_token: &str,
    ) -> Result<(), Error> {
        let subject = "Please confirm your registration".to_string();

        let text = format!(
            "Welcome to {}!\nEnter the code to confirm your registration: {}",
            config.app.name, register_token
        );
        let html = format!(
            "Welcome to {}!<br />\
        Enter the code to confirm your registration: {}",
            config.app.name, register_token
        );

        let payload = EmailPayload::new(
            email_client.sender.clone(),
            vec![email_recipient.to_owned()],
            subject,
            html,
            text,
        );

        email_client.send_email(payload).await
    }
}
