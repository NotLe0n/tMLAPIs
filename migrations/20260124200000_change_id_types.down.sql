ALTER TABLE mod_history
ALTER COLUMN mod_id TYPE TEXT USING mod_id::TEXT;

ALTER TABLE mod_history
ALTER COLUMN author_id TYPE TEXT USING author_id::TEXT;