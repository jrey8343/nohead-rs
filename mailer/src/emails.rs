use crate::{EmailClient, EmailPayload, Error};

pub async fn send_register_confirm_email(
    email_client: &EmailClient,
    email_recipient: &str,
    register_token: &str,
) -> Result<(), Error> {
    //FIX: WRONG BASE URL
    let confirmation_link = format!(
        "{}/auth/register/confirm?register_token={}",
        email_client.base_url, register_token
    );

    let subject = "Please confirm your registration".to_string();

    let text = format!(
        "Welcome to Nohead!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html = format!(
        "Welcome to Nohead!<br />\
        Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
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
