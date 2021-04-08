pub mod schema;

mod database;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate diesel;

use futures::executor::block_on;
use serenity::client::{EventHandler};
use serenity::model::guild::Guild;
use serenity::model::prelude::PartialGuild;
use std::collections::{LinkedList, HashSet};
use fancy_regex::{Regex, Match};

struct Spider;

impl EventHandler for Spider {
    // async fn guild_create(&self, ctx: Context, guild: Guild, is_new: bool) {
    //     println!("{}", guild.name);
    // }
}

async fn run_bot() {
    lazy_static! {
    static ref re: Regex = Regex::new(r#"(?<=discord.gg/)[a-z]+"#).unwrap();
}
    let token = "bfggobrrrr";
    let spider = Spider;
    let client = serenity::Client::new(&token, spider).unwrap();

    let mut servers: LinkedList<PartialGuild> = LinkedList::new();
    servers.push_back(
        Guild::get(&client.cache_and_http.http, 473760315293696010).expect("Seed guild not found."),
    );

    while let Some(guild) = servers.pop_back() {
        let msgs = client
            .cache_and_http
            .http
            .search_messages(guild.id.0, "discord.gg".to_string())
            .expect("Search failed.");

        let mut invites = HashSet::new();
        for msg in msgs {
            for invite in re.find_iter(&msg.content) {
                let invite: Match = invite.unwrap();
                invites.insert(invite.as_str().to_string());
            }
        }


        println!("{:#?}", invites);
    }
}

fn main() {
    block_on(run_bot());
}
