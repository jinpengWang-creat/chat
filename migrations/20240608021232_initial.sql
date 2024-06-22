-- Add migration script here

-- create user table
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    ws_id BIGINT NOT NULL,
    fullname VARCHAR(64) NOT NULL,
    email VARCHAR(64) NOT NULL UNIQUE,
    -- hashed argon2 password with 97 characters
    password_hash VARCHAR(97) NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- add a super user to altered users and set id to 0
INSERT INTO users (id, ws_id, fullname, email, password_hash)
VALUES (0, 0, 'super user', 'superuser@none.org', '');

-- add 5 users for testing
INSERT INTO users (ws_id, fullname, email, password_hash)
VALUES (0, 'test1', 'test1@none.org', ''),
       (0, 'test2', 'test2@none.org', ''),
       (0, 'test3', 'test3@none.org', ''),
       (0, 'test4', 'test4@none.org', ''),
       (0, 'test5', 'test5@none.org', '');

-- Add migration script here
CREATE TABLE workspaces (
    id bigserial PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    owner_id BIGINT NOT NULL REFERENCES users(id),
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- add a workspace for super user and set id to 0
INSERT INTO workspaces (id, name, owner_id)
VALUES (0, 'default', 0);

-- add foreign key constraint for ws_id to user
ALTER TABLE users
ADD CONSTRAINT fk_user_ws_id FOREIGN KEY (ws_id) REFERENCES workspaces(id);

-- create index for email
CREATE UNIQUE INDEX IF NOT EXISTS email_index ON users(email);

-- create chat type: single, group, private_channel, public_channel
CREATE TYPE chat_type AS ENUM ('single', 'group', 'private_channel', 'public_channel');

-- create chat table
CREATE TABLE IF NOT EXISTS chats (
    id BIGSERIAL PRIMARY KEY,
    ws_id BIGINT NOT NULL REFERENCES workspaces(id),
    name VARCHAR(64) ,
    type chat_type NOT NULL,
    -- use id list
    members BIGINT[] NOT NULL,
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- insert default chat for super user and 5 test users
INSERT INTO chats (id, ws_id, name, type, members)
VALUES (0, 0, 'default', 'group', '{0, 1, 2, 3, 4, 5}');

-- create message table
CREATE TABLE IF NOT EXISTS messages (
    id BIGSERIAL PRIMARY KEY,
    chat_id BIGINT NOT NULL REFERENCES chats(id),
    sender_id BIGINT NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    files TEXT[] DEFAULT '{}',
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- create index for message for chat_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS messages_chat_id_created_at_idx ON messages(chat_id, created_at DESC);

-- create index for members for sender_id and created_at order by created_at desc
CREATE INDEX IF NOT EXISTS sender_id_index ON messages(sender_id, created_at DESC);
