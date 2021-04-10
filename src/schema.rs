table! {
    guilds (snowflake) {
        snowflake -> Numeric,
        title -> Nullable<Text>,
        parent_invite -> Nullable<Text>,
    }
}

table! {
    invites (invite_code) {
        invite_code -> Text,
        origin_message_snowflake -> Numeric,
        guild_snowflake -> Nullable<Numeric>,
        queued -> Bool,
        recursion_level -> Int2,
    }
}

table! {
    messages (snowflake) {
        snowflake -> Numeric,
        channel_snowflake -> Nullable<Numeric>,
        guild_snowflake -> Nullable<Numeric>,
        author_snowflake -> Nullable<Numeric>,
        content -> Nullable<Text>,
    }
}

table! {
    users (snowflake) {
        snowflake -> Numeric,
        avatar -> Nullable<Text>,
        discriminator -> Int2,
        username -> Text,
        public_flags -> Int4,
    }
}

allow_tables_to_appear_in_same_query!(guilds, invites, messages, users,);
