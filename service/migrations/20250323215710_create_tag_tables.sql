CREATE TABLE tag (
    id UUID PRIMARY KEY, 
    description TEXT NOT NULL UNIQUE,
    account_id UUID references account(id)
);

CREATE TABLE image_tag (
    tag_id UUID references tag(id),
    image_id UUID references image(id),
    UNIQUE(tag_id, image_id)
);