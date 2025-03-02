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
        let confirmation_link = format!(
            "{}:{}/auth/register/confirm?register_token={}",
            config.server.host, config.server.port, register_token
        );

        let subject = "Please confirm your registration".to_string();

        let text = format!(
            "Welcome to {}!\nVisit {} to confirm your subscription.",
            config.app.name, confirmation_link
        );
        let html = format!(
            "Welcome to {}!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
            config.app.name, confirmation_link
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
