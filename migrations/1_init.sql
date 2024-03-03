CREATE TABLE "user"
(
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tg_id               INTEGER NOT NULL UNIQUE,
    username            TEXT    UNIQUE,
    first_name          TEXT    NOT NULL,
    last_name           TEXT,
    can_download        INTEGER NOT NULL,
    is_admin            INTEGER NOT NULL,
    has_private_chat    INTEGER NOT NULL
);

CREATE TABLE "chat"
(
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    tg_id               INTEGER NOT NULL UNIQUE,
    title               TEXT    NOT NULL,
    username            TEXT,
    can_download        INTEGER NOT NULL
);

CREATE TABLE "link"
(
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    domain              TEXT    NOT NULL UNIQUE,
    path                TEXT,
    download_allowed    INTEGER NOT NULL,
    auto_download       INTEGER NOT NULL
);

CREATE INDEX idx_link_domain
    ON "link"(domain);

CREATE TABLE "request"
(
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    requested_by        INTEGER NOT NULL UNIQUE,
    approved_by         INTEGER UNIQUE,
    message             TEXT,
    is_approved         INTEGER NOT NULL,

    FOREIGN KEY(requested_by)   REFERENCES "user"(id),
    FOREIGN KEY(approved_by)    REFERENCES "user"(id)
);

CREATE TRIGGER "approve"
AFTER UPDATE OF is_approved ON "request"
WHEN new.is_approved = 1
BEGIN
    UPDATE user SET can_download = 1 WHERE user.id = new.requested_by;
END;

CREATE TABLE "request_chat"
(
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    requested_by        INTEGER NOT NULL UNIQUE,
    requested_for       INTEGER NOT NULL UNIQUE,
    approved_by         INTEGER UNIQUE,
    message             TEXT,
    is_approved         INTEGER NOT NULL,

    FOREIGN KEY(requested_by)   REFERENCES "user"(id),
    FOREIGN KEY(requested_for)  REFERENCES "chat"(id),
    FOREIGN KEY(approved_by)    REFERENCES "user"(id)
);

CREATE TRIGGER "approve_chat"
AFTER UPDATE OF is_approved ON "request_chat"
WHEN new.is_approved = 1
BEGIN
    UPDATE chat SET can_download = 1 WHERE chat.id = new.requested_for;
END;