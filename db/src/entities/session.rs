use sqlx::prelude::FromRow;

#[derive(Clone, FromRow, Debug)]
pub struct Session {
    pub id: String,
    pub data: Vec<u8>,
    pub expiry_date: i64,
}
