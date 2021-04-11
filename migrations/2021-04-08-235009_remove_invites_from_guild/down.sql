-- This file should undo anything in `up.sql`
ALTER TABLE guilds ADD invite_codes TEXT[] NULL;