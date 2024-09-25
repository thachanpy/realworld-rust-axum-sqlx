CREATE TYPE USER_ROLE AS ENUM ('user', 'admin');
CREATE TYPE USER_STATUS AS ENUM ('registered', 'verified');
CREATE TYPE OAUTH2_PROVIDER AS ENUM ('google');

CREATE TABLE users
(
    id               UUID                     NOT NULL PRIMARY KEY DEFAULT uuid_generate_v4(),
    email            TEXT UNIQUE              NOT NULL,
    password         TEXT,
    name             TEXT,
    profile_url      TEXT,
    role             USER_ROLE,
    status           USER_STATUS,
    auth_id          TEXT,
    auth_provider    OAUTH2_PROVIDER,
    logged_in_at     TIMESTAMP WITH TIME ZONE,
    created_at       TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMP WITH TIME ZONE,
    deleted_at       TIMESTAMP WITH TIME ZONE
);

CREATE OR REPLACE FUNCTION users_updated_at() RETURNS trigger
    LANGUAGE plpgsql
AS
$$
BEGIN
    IF NEW IS DISTINCT FROM OLD
        AND NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
        AND NOT (NEW.logged_in_at IS DISTINCT FROM OLD.logged_in_at)
    THEN
        NEW.updated_at = NOW();
    END IF;
    RETURN NEW;
END;
$$;

CREATE OR REPLACE TRIGGER trigger_users_updated_at
    BEFORE UPDATE
    ON users
    FOR EACH ROW
EXECUTE PROCEDURE users_updated_at();
