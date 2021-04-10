use super::schema::{guilds, invites};
use bigdecimal::BigDecimal;
use diesel::{prelude::*, result::Error};
use diesel::{PgConnection, Queryable};
use dotenv::dotenv;
use std::collections::VecDeque;
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

#[derive(Queryable, Insertable)]
pub struct Guild {
    pub snowflake: BigDecimal,
    pub title: Option<String>,
    pub parent_invite: Option<String>,
}

#[derive(Queryable)]
pub struct User {
    snowflake: BigDecimal,
    avatar: Option<String>,
    discriminator: u8,
    username: String,
    public_flags: u32,
}

#[derive(Queryable, Insertable, Debug)]
pub struct Invite {
    pub invite_code: String,
    origin_message_snowflake: BigDecimal,
    guild_snowflake: Option<BigDecimal>,
    queued: bool,
    recursion_level: i16,
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Fetch message from DB by snowflake.
pub fn fetch_message_by_snowflake<T>(
    message_snowflake: &BigDecimal,
    conn: T,
) -> Result<Option<Message>, Error>
where
    T: Deref<Target = PgConnection>,
{
    use super::schema::messages::dsl::*;
    messages
        .filter(snowflake.eq(message_snowflake))
        .first::<Message>(&*conn)
        .optional()
}

/// Fetch guild from DB by snowflake.
pub fn fetch_guild_by_snowflake<T>(
    guild_snowflake: &BigDecimal,
    conn: T,
) -> Result<Option<Guild>, Error>
where
    T: Deref<Target = PgConnection>,
{
    use super::schema::guilds::dsl::*;
    guilds
        .filter(snowflake.eq(guild_snowflake))
        .first::<Guild>(&*conn)
        .optional()
}

/// Is this guild in the database ?
fn guild_in_database<T>(guild_snowflake: &BigDecimal, conn: T) -> Result<bool, Error>
where
    T: Deref<Target = PgConnection>,
{
    use super::schema::guilds::dsl::*;
    use diesel::dsl::{exists, select};

    Ok(select(exists(guilds.filter(snowflake.eq(guild_snowflake)))).get_result(&*conn)?)
}

/// Get invite in DB matching `invite_code`.  
/// Returns `Ok(None)` if not found, `Ok(Some(Invite))` if found, and `Err` if something else is broken.
pub fn fetch_invite_by_code<T>(invite_code: &str, conn: T) -> Result<Option<Invite>, Error>
where
    T: Deref<Target = PgConnection>,
{
    use super::schema::invites::dsl;

    dsl::invites
        .filter(dsl::invite_code.eq(invite_code))
        .first::<Invite>(&*conn)
        .optional()
}

pub fn get_recursion_level_of_message<T>(
    message_snowflake: &BigDecimal,
    conn: T,
) -> Result<Option<i16>, Error>
where
    T: Deref<Target = PgConnection>,
{
    let message = fetch_message_by_snowflake(message_snowflake, &*conn)?;
    if let Some(message) = message {
        if let Some(guild_snowflake) = message.guild_snowflake {
            let guild = fetch_guild_by_snowflake(&guild_snowflake, &*conn)?.unwrap_or_else(|| {
                panic!(
                    "Invalid parent guild {} for message {}.",
                    guild_snowflake, message_snowflake
                )
            });
            let parent_invite = guild.parent_invite.unwrap();
            Ok(Some(
                fetch_invite_by_code(&parent_invite, conn)?
                    .unwrap_or_else(|| {
                        panic!(
                            "Invalid parent invite {} for guild {}.",
                            parent_invite, guild_snowflake
                        )
                    })
                    .recursion_level,
            ))
        } else {
            Ok(Some(0))
        }
    } else {
        Ok(None)
    }
}

/// Put a guild in the database.
pub fn insert_guild<T>(guild: Guild, conn: T) -> Result<(), Error>
where
    T: Deref<Target = PgConnection>,
{
    use super::schema::guilds::dsl::*;

    let guild_in_db: bool = guild_in_database(&guild.snowflake, &*conn)?;

    if guild_in_db {
        Err(Error::AlreadyInTransaction)
    } else {
        diesel::insert_into(guilds)
            .values(guild)
            .execute(&*conn)
            .map(|_| ())
    }
}

/// Is this invitation link in the database ?
fn invite_in_database<T>(invite_code: &str, conn: T) -> Result<bool, Error>
where
    T: Deref<Target = PgConnection>,
{
    use super::schema::invites::dsl;
    use diesel::dsl::{exists, select};

    Ok(select(exists(
        dsl::invites.filter(dsl::invite_code.eq(invite_code)),
    ))
    .get_result(&*conn)?)
}

/// Just sticks an invite in the db.  Private because [insert_and_queue_invite_code] should be used.
fn insert_invite_code<T>(invite: Invite, conn: T) -> Result<(), Error>
where
    T: Deref<Target = PgConnection>,
{
    use super::schema::invites::dsl::*;
    use diesel::dsl::{exists, select};

    let invite_in_db: bool =
        select(exists(invites.filter(invite_code.eq(&invite.invite_code)))).get_result(&*conn)?;

    if invite_in_db {
        Err(Error::AlreadyInTransaction)
    } else {
        diesel::insert_into(invites)
            .values(invite)
            .execute(&*conn)
            .map(|_| ())
    }
}

/// Inserts & queues invite code.  Tries to work out if it should be queued by checking whether or not it already exists
pub fn insert_and_queue_invite_code<T>(
    invite_code: String,
    origin_message_snowflake: BigDecimal,
    guild_snowflake: Option<BigDecimal>,
    conn: T,
) -> Result<(), Error>
where
    T: Deref<Target = PgConnection>,
{
    let recursion_level = 1 + get_recursion_level_of_message(&origin_message_snowflake, &*conn)?
        .unwrap_or_else(|| panic!("Origin message {} not found in database for invite code {}", origin_message_snowflake, invite_code));

    let invite_in_database = invite_in_database(&invite_code, &*conn)?;

    insert_invite_code(
        Invite {
            invite_code,
            origin_message_snowflake,
            guild_snowflake,
            queued: !invite_in_database,
            recursion_level,
        },
        &*conn,
    )
}

/// Gets queued invites.
/// note: marks them as non-queued;  this will be a *fucking nightmare* when i try to scale, but whatever
pub fn get_queued_invites<T>(_recursion_level: i16, conn: T) -> Result<VecDeque<Invite>, Error>
where
    T: Deref<Target = PgConnection>,
{
    use super::schema::invites::dsl;
    use diesel::dsl::update;

    let invite_selection = dsl::invites
        .filter(dsl::queued);

    let results = invite_selection
    .load(&*conn)
    .map(VecDeque::from)?;

    if !results.is_empty() {
    println!("{:#?}", &results);
    }
    update(invite_selection)
        .set(dsl::queued.eq(false))
        .execute(&*conn)?;

    Ok(results)
}
