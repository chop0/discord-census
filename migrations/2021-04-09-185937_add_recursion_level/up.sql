-- Your SQL goes here
ALTER TABLE guilds ADD parent_invite TEXT;
ALTER TABLE invites ADD recursion_level SMALLINT NOT NULL;