-- Identity: who plays, who is signed in, and which accounts may not come back (spec rule 3.5.1).
-- The application writes every timestamp from its clock; no column defaults to now().

CREATE TABLE players (
    id BIGINT GENERATED ALWAYS AS IDENTITY CONSTRAINT players_pkey PRIMARY KEY,
    -- Google's stable ID for the account (spec rule 1.6.1).
    google_subject TEXT NOT NULL CONSTRAINT players_google_subject_unique UNIQUE,
    email TEXT NOT NULL,
    -- NULL until the player has chosen a name (spec rule 3.1.3).
    display_name TEXT,
    -- The name in the form that decides uniqueness (spec rule 1.6.2).
    -- The application computes it; Postgres only compares it.
    display_name_key TEXT CONSTRAINT players_display_name_key_unique UNIQUE,
    -- Google's name for the account, kept only to prefill the name step.
    suggested_name TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT players_name_and_key_together CHECK ((display_name IS NULL) = (display_name_key IS NULL)),
    CONSTRAINT players_suggestion_only_while_nameless CHECK (display_name IS NULL OR suggested_name IS NULL)
);

CREATE TABLE sessions (
    -- The SHA-256 hash of the session ID; the ID itself is never stored (spec rule 3.2.1).
    token_hash BYTEA CONSTRAINT sessions_pkey PRIMARY KEY
        CONSTRAINT sessions_token_hash_is_32_bytes CHECK (octet_length(token_hash) = 32),
    -- Deleting a player ends their sessions (spec rule 3.2.5).
    player_id BIGINT NOT NULL CONSTRAINT sessions_player_id_fkey REFERENCES players (id) ON DELETE CASCADE,
    -- A session expires 90 days after this instant (spec rule 3.2.4).
    last_used_at TIMESTAMPTZ NOT NULL
);

-- Finds a player's sessions, to end them all at once and to delete them with the player.
CREATE INDEX sessions_player_id ON sessions (player_id);

-- A removed player's Google account (spec rule 1.6.7).
CREATE TABLE blocked_accounts (
    google_subject TEXT CONSTRAINT blocked_accounts_pkey PRIMARY KEY,
    email TEXT NOT NULL,
    removed_at TIMESTAMPTZ NOT NULL
);
