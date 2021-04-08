-- Your SQL goes here
CREATE TABLE messages
(
    snowflake         NUMERIC,
    channel_snowflake NUMERIC,
    guild_snowflake   NUMERIC,
    author_snowflake  NUMERIC,
    content           TEXT,

    PRIMARY KEY (snowflake)
);

CREATE TABLE guilds
(
    snowflake    NUMERIC NOT NULL,
    title        TEXT    NULL,
    invite_codes TEXT[]  NULL,

    PRIMARY KEY (snowflake)
);

CREATE TABLE users
(
    snowflake     NUMERIC  NOT NULL,
    avatar        TEXT     NULL,
    discriminator SMALLINT NOT NULL,
    username      TEXT     NOT NULL,
    public_flags  INT      NOT NULL,

    PRIMARY KEY (snowflake)
);

CREATE TABLE invites
(
    invite_code              TEXT    NOT NULL,
    origin_message_snowflake NUMERIC NOT NULL,
    guild_snowflake          NUMERIC NULL,

    PRIMARY KEY (invite_code)
);