CREATE TABLE account (
    id UUID PRIMARY KEY, 
    username TEXT NOT NULL,
    password TEXT NOT NULL
);

CREATE TABLE token (
    id UUID PRIMARY KEY, 
    token TEXT NOT NULL, 
    account_id UUID references account(id)
);