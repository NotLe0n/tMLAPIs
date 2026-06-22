CREATE TABLE authors (
    author_id       BIGINT PRIMARY KEY,
    author_names    TEXT[] NOT NULL DEFAULT '{}',
    total_downloads BIGINT NOT NULL DEFAULT 0,
    total_views     BIGINT NOT NULL DEFAULT 0,
    total_favorited BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE author_mods (
    author_id       BIGINT NOT NULL REFERENCES authors(author_id) ON DELETE CASCADE,
    mod_id          BIGINT NOT NULL,
    display_name    TEXT NOT NULL,
    internal_name   TEXT NOT NULL,
    PRIMARY KEY (author_id, mod_id)
);
