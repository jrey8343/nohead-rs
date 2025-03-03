use nohead_rs_config::Config;

use crate::{EmailClient, EmailPayload};

pub struct AuthMailer;

impl AuthMailer {
    pub fn send_confirmation(
        email_client: &EmailClient,
        config: &Config,
        email_recipient: &str,
        register_token: &str,
    ) -> EmailPayload {
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

        EmailPayload::new(
            email_client.sender.clone(),
            vec![email_recipient.to_owned()],
            subject,
            html,
            text,
        )
    }
}
