CREATE TABLE IF NOT EXISTS contracts
(
    contract_address TEXT PRIMARY KEY,
    owner            TEXT NOT NULL,
    tnx_hash         TEXT NOT NULL,
    created_at       TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS users_info
(
    user_address  TEXT PRIMARY KEY,
    username      TEXT    NOT NULL,
    is_registered BOOLEAN NOT NULL,
    created_at    TEXT    NOT NULL,
    tnx_hash      TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS manufacturers
(
    manufacturer_address TEXT PRIMARY KEY,
    manufacturer_name    TEXT    NOT NULL,
    is_registered        BOOLEAN NOT NULL,
    registered_at        TEXT    NOT NULL,
    tnx_hash             TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS ownership_codes
(
    ownership_code TEXT PRIMARY KEY,
    item_id TEXT NOT NULL,
    item_owner     TEXT NOT NULL,
    temp_owner     TEXT NOT NULL,
    created_at     TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS items
(
    id           SERIAL PRIMARY KEY,
    item_id      TEXT   NOT NULL UNIQUE,
    name         TEXT   NOT NULL,
    serial       TEXT   NOT NULL,
    date         BIGINT NOT NULL,
    owner        TEXT   NOT NULL,
    manufacturer TEXT   NOT NULL,
    metadata     TEXT[] NOT NULL,
    created_at   TEXT   NOT NULL,
    tnx_hash     TEXT   NOT NULL
);

CREATE TABLE IF NOT EXISTS ownership_claims
(
    id         SERIAL PRIMARY KEY,
    item_id    TEXT NOT NULL,
    old_owner  TEXT NOT NULL,
    new_owner  TEXT NOT NULL,
    tnx_hash   TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS code_revokations
(
    id         SERIAL PRIMARY KEY,
    item_hash  TEXT NOT NULL,
    tnx_hash   TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS authenticity_settings
(
    id                   SERIAL PRIMARY KEY,
    authenticity_address TEXT NOT NULL,
    tnx_hash             TEXT NOT NULL,
    created_at           TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS certificates
(
    unique_id     TEXT PRIMARY KEY,
    name          TEXT   NOT NULL,
    serial        TEXT   NOT NULL,
    date          BIGINT NOT NULL,
    owner         TEXT   NOT NULL,
    metadata_hash TEXT   NOT NULL,
    metadata      TEXT[] NOT NULL,
    signature     TEXT   NOT NULL
);
