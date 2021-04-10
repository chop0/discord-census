#![allow(dead_code)]

pub mod schema;

mod database;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel;

use std::env;

use std::{collections::HashSet, sync::Arc};

use bigdecimal::{BigDecimal, FromPrimitive};
use database::{
    establish_connection, fetch_guild_by_snowflake, get_queued_invites,
    insert_and_queue_invite_code, insert_guild,
};
use diesel::{result::Error, PgConnection};
use fancy_regex::{Match, Regex};
use futures::executor::block_on;
use serenity::client::EventHandler;

use serenity::model::id::GuildId;

use bigdecimal::ToPrimitive;
use serenity::http::GuildPagination;

struct Spider;

impl EventHandler for Spider {
    // async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
    //     println!("{}", guild.name);
    // }
}

async fn run_bot(conn: Arc<PgConnection>) -> Result<(), Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(?<=discord.gg/)[a-z]+"#).unwrap();
    }
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");
    let spider = Spider;
    let client = serenity::Client::new(&token, spider).unwrap();

    /*
    servers.push_back(
        Guild::get(&client.cache_and_http.http, 473760315293696010).expect("Seed guild not found."),
    );
    */

    loop {
        let mut queued_invites =
            get_queued_invites(-1, conn.clone()).expect("Error accessing queued invites.");
        if !queued_invites.is_empty() {
            println!("{:#?}", queued_invites.len());
        }
        while let Some(invite) = queued_invites.pop_back() {
            println!("Observing server {}.", &invite.invite_code);
            // TODO: join guild if we need to and
            let guild_for_invite = client
                .cache_and_http
                .http
                .get_invite(&invite.invite_code, false);

            let guild_for_invite = if let Ok(res) = guild_for_invite {
                if let Some(guild) = res.guild {
                    BigDecimal::from(guild.id.0)
                } else {
                    continue;
                }
            } else {
                continue;
            };

            // if client
            //     .cache_and_http
            //     .http
            //     .get_guilds(&GuildPagination::After(GuildId(0)), 100)
            //     .expect("Error getting current guilds.")
            //     .iter()
            //     .map(|x| BigDecimal::from(x.id.0))
            //     .any(|x| x == guild_for_invite)
            // {
            //     println!("INFO:  Cycle detected starting in {}.", guild_for_invite);
            //     continue;
            // }

            if let Ok(Some(_)) = fetch_guild_by_snowflake(&guild_for_invite, conn.clone()) {
                println!("INFO:  Already visited guild {}.", guild_for_invite);
                continue;
            }

            client
                .cache_and_http
                .http
                .join_guild(&invite.invite_code)
                .unwrap_or_else(|_| {
                    panic!(
                        "Error joining guild {} with invite link https://discord.gg/{}.",
                        guild_for_invite, &invite.invite_code
                    )
                });

            let msgs = client
                .cache_and_http
                .http
                .search_messages(guild_for_invite.to_u64().unwrap(), "discord.gg".to_string())
                .expect("Search failed.");

            for msg in msgs {
                for invite in RE.find_iter(&msg.content) {
                    let invite: Match = invite.unwrap();
                    insert_and_queue_invite_code(
                        invite.as_str().to_string(),
                        BigDecimal::from(msg.id.0),
                        Some(BigDecimal::from(msg.guild_id.unwrap().0)),
                        conn.clone(),
                    )
                    .unwrap_or_else(|_| {
                        panic!("Error queuing invite {}.", invite.as_str().to_string())
                    });
                }
            }

            client
                .cache_and_http
                .http
                .leave_guild(guild_for_invite.to_u64().unwrap())
                .unwrap_or_else(|_| panic!("Error leaving guild {}.", guild_for_invite));
        }
    }
}

fn main() {
    let conn = establish_connection();
    // insert_guild(
    //     database::Guild {
    //         snowflake: BigDecimal::from_u64(69).unwrap(),
    //         title: Some("Lemon Lounge".to_string()),
    //         parent_invite: None,
    //     },
    //     &conn,
    // )
    // .unwrap();

    block_on(run_bot(Arc::new(conn)));
}
