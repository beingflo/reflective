CREATE TABLE image (
    id UUID PRIMARY KEY, 
    filename TEXT NOT NULL,
    captured_at TEXT NOT NULL,
    metadata TEXT,
    account_id UUID references account(id)
);

CREATE TABLE variant (
    id UUID PRIMARY KEY, 
    object_name TEXT NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    compression_quality INTEGER NOT NULL,
    quality TEXT NOT NULL, 
    version INTEGER NOT NULL,
    image_id UUID references image(id)
);