use apalis::prelude::*;
use nohead_rs_mailer::EmailPayload;

use crate::error::Error;
use crate::state::AppState;

pub async fn send_email(job: EmailPayload, app_state: Data<AppState>) -> Result<(), Error> {
    app_state.email_client.send_email(job).await?;

    Ok(())
}
