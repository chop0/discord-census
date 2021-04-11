use serde::Deserialize;
use std::env;

use crate::util::deserialize_number_from_string;

macro_rules! add_humanlike_imperfection {
    ($thing_to_make_non_sus:expr) => {
        $thing_to_make_non_sus
        .header("accept", "*/*")
        .header("accept-language", "en-US")
        .header("cache-control", "no-cache")
        .header("cookie", "locale=en-US; __dcfduid=2fe9e5328afd33729c2b55f26e7dfdf2; __cfduid=d2457bc40abddaf2fb3bf6a5b1830ad881618091818")
        .header("origin", "https://discord.com")
        .header("pragma", "no-cache")
        .header("user-agent", "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) discord/0.0.309 Chrome/83.0.4103.122 Electron/9.3.5 Safari/537.36")
    };
}

// TODO: make this not awful
#[derive(Deserialize)]
struct __RawSearchResult {
    messages: Vec<Vec<DiscordMessage>>,
}

impl __RawSearchResult {
    pub fn get_results(self) -> Vec<DiscordMessage> {
        self.messages.into_iter().flatten().collect()
    }
}
#[derive(Deserialize, Debug)]
pub struct DiscordMessage {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: u64,
    pub content: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub channel_id: u64,
    pub author: DiscordUser,
}

#[derive(Deserialize, Debug)]
pub struct DiscordUser {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: u64,
    pub username: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub discriminator: u16,
    pub avatar: Option<String>,
    pub public_flags: Option<u32>,
    pub bot: Option<bool>
}

#[derive(Deserialize, Debug)]
pub struct DiscordGuild {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub id: u64,
    pub name: String,
}
#[derive(Deserialize, Debug)]
pub struct DiscordInvite {
    pub code: String,
    pub guild: DiscordGuild,
}

pub struct DiscordClient {
    api_url: &'static str,
    token: String,
}

impl DiscordClient {
    pub async fn get_invite_details(&self, invite: &str) -> reqwest::Result<DiscordInvite> {
        let client = reqwest::ClientBuilder::new().build().unwrap();

        add_humanlike_imperfection!(client.get(format!("{}/invites/{}", self.api_url, invite)))
            .header("authorization", &self.token)
            .send()
            .await?
            .json::<DiscordInvite>()
            .await
    }

    pub async fn join_guild(&self, invite: &str) -> reqwest::Result<()> {
        let client = reqwest::ClientBuilder::new().build().unwrap();
        add_humanlike_imperfection!(client.post(format!("{}/invites/{}", self.api_url, invite)))
            .header("authorization", &self.token)
            .send()
            .await
            .map(|_| ())
    }

    pub async fn leave_guild(&self, guild_id: u64) -> reqwest::Result<()> {
        let client = reqwest::ClientBuilder::new().build().unwrap();

        add_humanlike_imperfection!(
            client.delete(format!("{}/users/@me/guilds/{}", self.api_url, guild_id))
        )
        .header("authorization", &self.token)
        .send()
        .await
        .map(|_| ())
    }

    pub async fn search_messages(
        &self,
        guild_id: u64,
        content: &str,
    ) -> reqwest::Result<Vec<DiscordMessage>> {
        let client = reqwest::ClientBuilder::new().build().unwrap();

        let response = add_humanlike_imperfection!(client.get(format!(
            "{}/guilds/{}/messages/search?content={}",
            self.api_url, guild_id, content
        )))
        .header("authorization", &self.token)
        .send()
        .await?;
     //   println!("Status: {}\n, {:#?}", response.status(), response.text().await);


      //  Ok(vec![])
       response
            .json::<__RawSearchResult>()
           .await
            .map(|raw_result| raw_result.get_results())
    }
}

impl Default for DiscordClient {
    fn default() -> Self {
        Self {
            api_url: "https://discord.com/api/v8",
            token: env::var("DISCORD_TOKEN").expect("Default: DISCORD_TOKEN must be set"),
        }
    }
}
