use apalis::prelude::Data;
use nohead_rs_mailer::{EmailClient, EmailPayload};

pub async fn send_email(
    job: EmailPayload,
    email_client: Data<EmailClient>,
) -> Result<(), nohead_rs_mailer::Error> {
    email_client.send_email(job).await?;

    Ok(())
}
