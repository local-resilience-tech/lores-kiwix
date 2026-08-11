-- Projection schema for lores-websites.
--
-- This is the single source of truth for the projection database schema.
-- Edit this file freely — the framework detects changes via a content hash
-- and will drop and rebuild the database, then replay all operations.

CREATE TABLE zims (
    id          TEXT PRIMARY KEY
);
