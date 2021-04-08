use bigdecimal::BigDecimal;
use diesel::{prelude::*, result::Error};
use diesel::{PgConnection, Queryable};
use dotenv::dotenv;
use std::env;
use std::ops::Deref;

#[derive(Queryable)]
pub struct Message {
    snowflake: BigDecimal,
    channel_snowflake: Option<BigDecimal>,
    guild_snowflake: Option<BigDecimal>,
    author_snowflake: Option<BigDecimal>,
    content: Option<String>,
}

#[derive(Queryable)]
pub struct Guild {
    snowflake: BigDecimal,
    title: Option<String>,
    invite_codes: Option<Vec<String>>,
}

#[derive(Queryable)]
pub struct User {
    snowflake: BigDecimal,
    avatar: Option<String>,
    discriminator: u8,
    username: String,
    public_flags: u32,
}

#[derive(Queryable)]
pub struct Invite {
    invite_code: String,
    origin_message_snowflake: BigDecimal,
    guild_snowflake: Option<BigDecimal>,
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

fn fetch_message_by_snowflake<T: Deref<Target = PgConnection>>(
    conn: T,
    _snowflake: BigDecimal,
) -> Result<Option<Message>, Error> {
    use super::schema::messages::dsl::*;
    messages
        .filter(snowflake.eq(snowflake))
        .first::<Message>(&*conn)
        .optional()
}
