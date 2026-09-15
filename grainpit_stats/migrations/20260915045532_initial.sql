CREATE TABLE IF NOT EXISTS account (
    id SERIAL PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    key UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    grainpit_urls TEXT [] NOT NULL DEFAULT array []::text []
);
CREATE TABLE IF NOT EXISTS request (
    time TIMESTAMPTZ NOT NULL,
    creator INTEGER NOT NULL REFERENCES account (id),
    url TEXT NOT NULL,
    ip INET NOT NULL,
    user_agent TEXT NOT NULL
) WITH (tsdb.hypertable);