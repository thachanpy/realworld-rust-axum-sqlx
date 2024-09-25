CREATE TABLE refresh_tokens
(
    id         UUID                     NOT NULL PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id    UUID                     NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS index_refresh_tokens_by_user_id
    ON refresh_tokens (user_id);
