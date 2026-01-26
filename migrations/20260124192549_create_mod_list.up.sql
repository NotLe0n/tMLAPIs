CREATE TABLE mods (
    mod_id              BIGINT PRIMARY KEY,
    display_name        TEXT NOT NULL,
    internal_name       TEXT NOT NULL,
    author              TEXT NOT NULL,
    author_id           BIGINT NOT NULL,
    modside             TEXT NOT NULL,
    homepage            TEXT NOT NULL,
    mod_references      TEXT NOT NULL,
    num_versions        INTEGER NOT NULL,
    time_created        BIGINT NOT NULL,
    time_updated        BIGINT NOT NULL,
    workshop_icon_url   TEXT NOT NULL,
    description         TEXT,
    downloads_total     INTEGER NOT NULL CHECK (downloads_total >= 0),
    favorited           INTEGER NOT NULL CHECK (favorited >= 0),
    followers           INTEGER NOT NULL CHECK (followers >= 0),
    views               BIGINT NOT NULL CHECK (views >= 0),
    playtime            TEXT NOT NULL,
    num_comments        INTEGER NOT NULL CHECK (num_comments >= 0),
    score               DOUBLE PRECISION NOT NULL,
    votes_up            INTEGER NOT NULL CHECK (votes_up >= 0),
    votes_down          INTEGER NOT NULL CHECK (votes_down >= 0)
);

CREATE INDEX idx_mods_internal_name ON mods (internal_name);
CREATE INDEX idx_mod_score ON mods (score DESC);

CREATE TABLE mod_versions (
    id                      SERIAL PRIMARY KEY,
    mod_id                  BIGINT NOT NULL REFERENCES mods(mod_id) ON DELETE CASCADE,
    mod_version             TEXT NOT NULL,
    tmodloader_version      TEXT NOT NULL
);

CREATE TABLE mod_socials (
    mod_id      BIGINT PRIMARY KEY REFERENCES mods(mod_id) ON DELETE CASCADE,
    youtube     TEXT,
    twitter     TEXT,
    reddit      TEXT,
    facebook    TEXT,
    sketchfab   TEXT
);

CREATE TABLE mod_children (
    parent_mod_id   BIGINT NOT NULL REFERENCES mods(mod_id) ON DELETE CASCADE,
    child_mod_id    BIGINT NOT NULL REFERENCES mods(mod_id) ON DELETE CASCADE,

    CONSTRAINT mod_children_pk PRIMARY KEY (parent_mod_id, child_mod_id),

    CONSTRAINT mod_children_no_self_reference CHECK (parent_mod_id <> child_mod_id)
);

CREATE INDEX idx_mod_children_parent ON mod_children (parent_mod_id);

CREATE INDEX idx_mod_children_child ON mod_children (child_mod_id);


CREATE TABLE mod_tags (
    id              SERIAL PRIMARY KEY,
    mod_id          BIGINT NOT NULL REFERENCES mods(mod_id) ON DELETE CASCADE,
    tag             TEXT NOT NULL,
    display_name    TEXT NOT NULL
);

