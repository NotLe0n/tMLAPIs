ALTER TABLE mods
  RENAME COLUMN downloads_total TO subscriptions;

ALTER TABLE mods
  ADD COLUMN subscriptions_total INTEGER NOT NULL DEFAULT 0;

ALTER TABLE mods
  ADD COLUMN favorited_total INTEGER NOT NULL DEFAULT 0;

ALTER TABLE mods
  ADD COLUMN sessions INTEGER NOT NULL DEFAULT 0;

ALTER TABLE mods
  ADD COLUMN file_size TEXT NOT NULL DEFAULT '0';

ALTER TABLE mod_history
  RENAME COLUMN downloads_total TO subscriptions;

ALTER TABLE mod_history
  ADD COLUMN sessions INTEGER NOT NULL DEFAULT 0;

ALTER TABLE authors
  RENAME COLUMN total_downloads TO total_subscriptions;