ALTER TABLE mods
  RENAME COLUMN subscriptions TO downloads_total;

ALTER TABLE mods
  DROP COLUMN IF EXISTS file_size;

ALTER TABLE mods
  DROP COLUMN IF EXISTS sessions;

ALTER TABLE mods
  DROP COLUMN IF EXISTS favorited_total;

ALTER TABLE mods
  DROP COLUMN IF EXISTS subscriptions_total;

ALTER TABLE mod_history
  RENAME COLUMN subscriptions TO downloads_total;

ALTER TABLE mod_history
  DROP COLUMN IF EXISTS sessions;

ALTER TABLE authors
  RENAME COLUMN total_subscriptions TO total_downloads;