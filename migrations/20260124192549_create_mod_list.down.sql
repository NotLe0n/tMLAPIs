DROP INDEX IF EXISTS idx_mod_votes_score;
DROP INDEX IF EXISTS idx_mod_children_parent;
DROP INDEX IF EXISTS idx_mod_children_child;
DROP INDEX IF EXISTS idx_mods_internal_name;

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS mod_votes CASCADE;
DROP TABLE IF EXISTS mod_tags CASCADE;
DROP TABLE IF EXISTS mod_children CASCADE;
DROP TABLE IF EXISTS mod_socials CASCADE;
DROP TABLE IF EXISTS mod_versions CASCADE;
DROP TABLE IF EXISTS mods CASCADE;
