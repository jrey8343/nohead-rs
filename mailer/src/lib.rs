pub mod auth;

use core::time;

use nohead_rs_config::MailerConfig;
use reqwest::Client;
use serde::Serialize;
use validator::{Validate, ValidationError};

#[derive(Serialize, Validate)]
pub struct EmailPayload {
    #[validate(email(message = "must be a valid email address"))]
    from: String,
    #[validate(custom(function = "validate_collection_of_emails"))]
    to: Vec<String>,
    #[validate(length(min = 1, message = "must be at least 1 character"))]
    subject: String,
    html: String,
    text: String,
}

fn validate_collection_of_emails(emails: &Vec<String>) -> Result<(), ValidationError> {
    for email in emails {
        if !validator::ValidateEmail::validate_email(&email) {
            return Err(ValidationError::new("must be a valid email address"));
        }
    }
    Ok(())
}

impl EmailPayload {
    pub fn new(
        from: String,
        #[cfg(debug_assertions)] mut to: Vec<String>,
        // mutable in debug mode to allow changing
        // the recipient
        #[cfg(not(debug_assertions))] to: Vec<String>,
        subject: String,
        html: String,
        text: String,
    ) -> Self {
        if cfg!(debug_assertions) {
            // Change the recipient to a test email address if we are in debug mode
            let test_recipient = "delivered@resend.dev".to_string();
            to = vec![test_recipient];
        };

        Self {
            from,
            to,
            subject,
            html,
            text,
        }
    }
}

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: String,
    authorization_token: String,
}

// Manual implementation of Debug for EmailClient to redact the authorization token
impl std::fmt::Debug for EmailClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmailClient")
            .field("http_client", &self.http_client)
            .field("base_url", &self.base_url)
            .field("sender", &self.sender)
            .field("authorization_token", &"[redacted")
            .finish()
    }
}

impl EmailClient {
    pub fn new(config: &MailerConfig) -> Self {
        let timeout = time::Duration::from_millis(config.timeout);
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        let authorization_token =
            std::env::var("RESEND_API_KEY").expect("RESEND_API_KEY must be set in .env");

        Self {
            http_client,
            base_url: config.base_url.clone(),
            sender: config.sender.clone(),
            authorization_token,
        }
    }

    pub async fn send_email(&self, payload: EmailPayload) -> Result<(), Error> {
        payload.validate()?;

        let url = format!("{}/emails", self.base_url);

        let res = self
            .http_client
            .post(url)
            .header(
                "Authorization",
                format!("Bearer {}", self.authorization_token),
            )
            .json(&payload)
            .send()
            .await?;

        res.error_for_status()?; // return an error if the response status is not 2xx

        Ok(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // An invalid input was attempted on the email client
    #[error("validation failed")]
    Validation(#[from] validator::ValidationErrors),
    // A reqwest error occurred
    #[error("reqwest error")]
    Request(#[from] reqwest::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{
        Dummy, Fake, Faker,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };

    use nohead_rs_config::{Config, Environment, load_config};
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("from").is_some()
                    && body.get("to").is_some()
                    && body.get("subject").is_some()
                    && body.get("html").is_some()
                    && body.get("text").is_some()
            } else {
                false
            }
        }
    }
    /// ------------------------------------------------------------------------
    /// Manual impl Dummy to allow dummy Vec for to field
    /// ------------------------------------------------------------------------
    /// # Usage
    /// ```rust
    /// let payload: EmailPayload = Faker.fake();
    /// ```
    /// ------------------------------------------------------------------------
    impl Dummy<Faker> for EmailPayload {
        fn dummy_with_rng<R: fake::Rng + ?Sized>(_: &Faker, rng: &mut R) -> Self {
            Self {
                from: SafeEmail().fake_with_rng(rng),
                to: vec![SafeEmail().fake_with_rng(rng)],
                subject: Sentence(1..2).fake_with_rng(rng),
                html: Paragraph(1..10).fake_with_rng(rng),
                text: Paragraph(1..10).fake_with_rng(rng),
            }
        }
    }

    fn get_test_email_client(mock_server: &MockServer) -> EmailClient {
        let config: Config = load_config(&Environment::Test).unwrap();

        let mailer_config = MailerConfig {
            base_url: mock_server.uri().to_string(),
            sender: config.mailer.sender.clone(),
            timeout: config.mailer.timeout,
        };
        EmailClient::new(&mailer_config)
    }

    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = get_test_email_client(&mock_server);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let payload: EmailPayload = Faker.fake();
        let result = email_client.send_email(payload).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let email_client = get_test_email_client(&mock_server);

        Mock::given(header_exists("Authorization"))
            .and(header("Content-Type", "application/json"))
            .and(path("/emails"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let payload: EmailPayload = Faker.fake();
        let _ = email_client.send_email(payload).await;
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_400() {
        let mock_server = MockServer::start().await;
        let email_client = get_test_email_client(&mock_server);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(400))
            .expect(1)
            .mount(&mock_server)
            .await;

        let payload: EmailPayload = Faker.fake();
        let result = email_client.send_email(payload).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let email_client = get_test_email_client(&mock_server);

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(100)))
            .expect(1)
            .mount(&mock_server)
            .await;

        let payload: EmailPayload = Faker.fake();
        let result = email_client.send_email(payload).await;

        assert!(result.is_err());
    }
}
