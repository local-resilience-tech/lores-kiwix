-- Projection schema for lores-websites.
--
-- This is the single source of truth for the projection database schema.
-- Edit this file freely — the framework detects changes via a content hash
-- and will drop and rebuild the database, then replay all operations.

CREATE TABLE books (
    id          TEXT PRIMARY KEY NOT NULL,
    filename    TEXT NOT NULL,
    name        TEXT NOT NULL,
    date        TEXT NOT NULL,
    flavour     TEXT NOT NULL,
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    language    TEXT NOT NULL,
    creator     TEXT NOT NULL,
    publisher   TEXT NOT NULL,
    category    TEXT NOT NULL,
    tags        TEXT NOT NULL,
    query_text  TEXT NOT NULL
);

-- Maps books to the nodes that hold them (many-to-many).
CREATE TABLE holdings (
    book_id TEXT NOT NULL,
    node_id TEXT NOT NULL,
    PRIMARY KEY (book_id, node_id),
    FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE
    FOREIGN KEY (node_id) REFERENCES nodes (id) ON DELETE CASCADE
);

CREATE INDEX idx_holdings_node ON holdings (node_id);

CREATE TABLE nodes (
    id          TEXT PRIMARY KEY NOT NULL,
    local       BOOLEAN NOT NULL DEFAULT FALSE
);