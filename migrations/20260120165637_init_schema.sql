-- Enable TimescaleDB
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Daily stats (time-series)
CREATE TABLE mod_history (
    mod_id TEXT,
    date DATE NOT NULL,
    downloads_total INTEGER,
    views BIGINT,
    followers INTEGER,
    favorited INTEGER,
    votes_up INTEGER,
    votes_down INTEGER,
    score DOUBLE PRECISION,
    num_comments INTEGER,
    playtime BIGINT,
    time_updated BIGINT,
    version TEXT,
    PRIMARY KEY (mod_id, date)
);

-- Convert to hypertable
SELECT create_hypertable(
    'mod_history',
    'date',
    if_not_exists => TRUE
);
