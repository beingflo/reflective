CREATE TABLE account (
    id SERIAL PRIMARY KEY, 
    username TEXT NOT NULL,
    password TEXT NOT NULL
);

CREATE TABLE token (
    id SERIAL PRIMARY KEY, 
    token TEXT NOT NULL, 
    account_id INTEGER references account(id)
);