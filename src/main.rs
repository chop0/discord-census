#![allow(dead_code)]

pub mod schema;

mod database;
mod discord;
mod util;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel;

use database::{
    establish_connection, fetch_guild_by_snowflake, get_queued_invites,
    insert_and_queue_invite_code, remove_invite_from_queue, set_guild_for_invite,
};
use diesel::{result::Error, PgConnection};
use discord::DiscordClient;
use fancy_regex::{Match, Regex};
use std::sync::Arc;

use bigdecimal::ToPrimitive;

macro_rules! if_error_continue {
    ($value:expr) => {
        if let Ok(res) = $value {
            res
        } else {
            continue;
        }
    };
}

async fn run_bot(conn: Arc<PgConnection>) -> Result<(), Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(?<=discord.gg/)[a-zA-Z0-9]+"#).unwrap();
    }
    let cli = DiscordClient::default();

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
            let guild_invite_belongs_to = cli.get_invite_details(&invite.invite_code).await;

            let guild_invite_belongs_to = if_error_continue!(guild_invite_belongs_to).guild.id;

            set_guild_for_invite(&invite.invite_code, guild_invite_belongs_to, conn.clone())
                .unwrap_or_else(|_| {
                    panic!(
                        "Could not set guild to {} for invite {}.",
                        &guild_invite_belongs_to, &invite.invite_code
                    )
                });

            if let Ok(Some(_)) = fetch_guild_by_snowflake(guild_invite_belongs_to, conn.clone()) {
                println!("INFO:  Already visited guild {}.", guild_invite_belongs_to);
                continue;
            }

            cli.join_guild(&invite.invite_code)
                .await
                .unwrap_or_else(|_| {
                    panic!(
                        "Error joining guild {} with invite link https://discord.gg/{}.",
                        guild_invite_belongs_to, &invite.invite_code
                    )
                });

            let msgs = if_error_continue!(
                cli.search_messages(guild_invite_belongs_to.to_u64().unwrap(), "discord.gg")
                    .await
            );

            for msg in msgs {
                for invite in RE.find_iter(&msg.content) {
                    let invite: Match = invite.unwrap();

                    println!(
                        "Found invite in guild {}: {:#?}.",
                        guild_invite_belongs_to, invite
                    );

                    if let Err(error) = insert_and_queue_invite_code(
                        invite.as_str().to_string(),
                        msg.id,
                        None,
                        conn.clone(),
                    ) {
                        if let Error::AlreadyInTransaction = error {
                            continue;
                        } else {
                            panic!(
                                "Error queuing invite {}. {:#?}",
                                invite.as_str().to_string(),
                                error
                            );
                        }
                    };

                    if let Err(err) = remove_invite_from_queue(invite.as_str(), conn.clone()) {
                        println!(
                            "Error removing invite {} from queue: {}.",
                            invite.as_str(),
                            err
                        );
                    }
                }
            }

            cli.leave_guild(guild_invite_belongs_to.to_u64().unwrap())
                .await
                .unwrap_or_else(|_| panic!("Error leaving guild {}.", guild_invite_belongs_to));
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    //cli.get_invite_details("hackthebox").await.unwrap();
    let conn = establish_connection();
    run_bot(Arc::new(conn)).await.unwrap();

    // let cli = DiscordClient::default();
    // println!("{:#?}", cli.search_messages(473760315293696010, "discord.gg").await.unwrap());
}
