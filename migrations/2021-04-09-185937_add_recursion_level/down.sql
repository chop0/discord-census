-- This file should undo anything in `up.sql`
ALTER TABLE guilds DROP COLUMN parent_invite;
ALTER TABLE invites DROP COLUMN recursion_level;