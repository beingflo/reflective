CREATE TABLE tag (
    id SERIAL PRIMARY KEY, 
    description TEXT NOT NULL UNIQUE,
    account_id INTEGER references account(id)
);

CREATE TABLE image_tag (
    tag_id INTEGER references tag(id),
    image_id TEXT references image(id),
    UNIQUE(tag_id, image_id)
);