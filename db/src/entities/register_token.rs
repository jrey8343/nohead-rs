#[cfg(feature = "test-helpers")]
use fake::Dummy;

use rand::Rng as _;
use serde::{Deserialize, Serialize};
use sqlx::{Sqlite, prelude::FromRow, types::time::OffsetDateTime};
use validator::Validate;

use crate::Error;

#[derive(Clone, FromRow)]
pub struct RegisterToken {
    pub register_token: String,
    pub user_id: i64,
    pub expires_at: Option<OffsetDateTime>,
}

#[derive(Deserialize, Validate, Clone)]
#[cfg_attr(feature = "test-helpers", derive(Serialize, Dummy))]
pub struct RegisterTokenValidate {
    /// The register token must be exactly 6 characters long.
    #[cfg_attr(feature = "test-helpers", dummy(expr = "generate_register_token()"))]
    #[validate(length(min = 6, max = 6, message = "token must be 6 characters long"))]
    pub register_token: String,
}

impl RegisterToken {
    pub async fn try_get_user_id_by_register_token(
        register_token: RegisterTokenValidate,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<Option<i64>, Error> {
        register_token.validate()?;
        let maybe_user_id = sqlx::query!(
            r#"SELECT user_id FROM registration_tokens WHERE register_token = ?

"#,
            register_token.register_token
        )
        .fetch_optional(executor)
        .await?
        .map(|row| row.user_id);

        Ok(maybe_user_id)
    }

    pub async fn create<'a>(
        user_id: i64,
        executor: impl sqlx::Executor<'_, Database = Sqlite>,
    ) -> Result<RegisterToken, Error> {
        let rand_token = generate_register_token();
        let register_token = sqlx::query_as!(
            RegisterToken,
            r#"INSERT into registration_tokens (register_token, user_id) values (
                $1, $2
            ) returning *

            "#,
            rand_token,
            user_id
        )
        .fetch_one(executor)
        .await?;

        Ok(register_token)
    }
}

fn generate_register_token() -> String {
    let mut rng = rand::rng();
    std::iter::repeat_with(|| rng.sample(rand::distr::Alphanumeric))
        .map(char::from)
        .take(6)
        .collect()
}
