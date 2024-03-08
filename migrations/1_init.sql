CREATE TABLE "user"
(
    id                  SERIAL  PRIMARY KEY,
    tg_id               BIGINT  NOT NULL UNIQUE,
    username            VARCHAR UNIQUE,
    first_name          VARCHAR NOT NULL,
    last_name           VARCHAR,
    can_download        BOOLEAN NOT NULL,
    is_admin            BOOLEAN NOT NULL,
    has_private_chat    BOOLEAN NOT NULL
);

CREATE INDEX idx_user_tg_id
    ON "user"(tg_id);

CREATE FUNCTION set_admin()
RETURNS TRIGGER AS $$
BEGIN
    IF new.is_admin THEN
        UPDATE "user" SET can_download = true WHERE "user".id = new.id;
    END IF;
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_admin
AFTER UPDATE OF is_admin ON "user"
FOR EACH ROW
EXECUTE FUNCTION set_admin();

CREATE TABLE "chat"
(
    id                  SERIAL  PRIMARY KEY,
    tg_id               BIGINT  NOT NULL UNIQUE,
    title               VARCHAR NOT NULL,
    username            VARCHAR,
    can_download        BOOLEAN NOT NULL
);

CREATE INDEX idx_chat_tg_id
    ON "chat"(tg_id);

CREATE TABLE "link"
(
    id                  SERIAL  PRIMARY KEY,
    domain              VARCHAR NOT NULL,
    path                VARCHAR,
    download_allowed    BOOLEAN NOT NULL,
    auto_download       BOOLEAN NOT NULL
);

CREATE INDEX idx_link_domain_path
    ON "link"(domain, path);

CREATE TABLE "request"
(
    id                  SERIAL  PRIMARY KEY,
    requested_by        INTEGER NOT NULL UNIQUE,
    approved_by         INTEGER UNIQUE,
    message             VARCHAR NOT NULL,
    is_approved         BOOLEAN NOT NULL,

    FOREIGN KEY(requested_by)   REFERENCES "user"(id),
    FOREIGN KEY(approved_by)    REFERENCES "user"(id)
);

CREATE FUNCTION approve()
RETURNS TRIGGER AS $$
BEGIN
    IF new.is_approved THEN
        UPDATE "user" SET can_download = true WHERE "user".id = new.requested_by;
    END IF;
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER approve
AFTER UPDATE OF is_approved ON request
FOR EACH ROW
EXECUTE FUNCTION approve();

CREATE TABLE "request_chat"
(
    id                  SERIAL  PRIMARY KEY,
    requested_by        INTEGER NOT NULL UNIQUE,
    requested_for       INTEGER NOT NULL UNIQUE,
    approved_by         INTEGER UNIQUE,
    message             VARCHAR NOT NULL,
    is_approved         BOOLEAN NOT NULL,

    FOREIGN KEY(requested_by)   REFERENCES "user"(id),
    FOREIGN KEY(requested_for)  REFERENCES "chat"(id),
    FOREIGN KEY(approved_by)    REFERENCES "user"(id)
);

CREATE FUNCTION approve_chat()
RETURNS TRIGGER AS $$
BEGIN
    IF new.is_approved THEN
        UPDATE "chat" SET can_download = true WHERE "chat".id = new.requested_for;
    END IF;
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER approve_chat
AFTER UPDATE OF is_approved ON request_chat
FOR EACH ROW
EXECUTE FUNCTION approve_chat();