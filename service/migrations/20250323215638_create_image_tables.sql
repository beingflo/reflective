CREATE TABLE image (
    id TEXT PRIMARY KEY, 
    filename TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    metadata TEXT,
    account_id INTEGER references account(id)
);

CREATE TABLE variant (
    id SERIAL PRIMARY KEY, 
    object_name TEXT NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    compression_quality INTEGER NOT NULL,
    quality TEXT NOT NULL, 
    version INTEGER NOT NULL,
    image_id TEXT references image(id)
);