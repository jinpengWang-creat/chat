-- Add migration script here
CREATE TABLE workspaces (
    id bigserial PRIMARY KEY,
    name VARCHAR(32) NOT NULL UNIQUE,
    owner_id BIGINT NOT NULL REFERENCES users(id),
    created_at timestamptz DEFAULT CURRENT_TIMESTAMP
);

-- alter table user to add ws_id column
ALTER TABLE users
ADD COLUMN ws_id BIGINT REFERENCES workspaces(id);


-- add a super user to altered users and set id to 0
INSERT INTO users (id, fullname, email, password_hash)
VALUES (0, 'super user', 'superuser@none.org', '');

-- add a workspace for super user and set id to 0
INSERT INTO workspaces (id, name, owner_id)
VALUES (0, 'default', 0);

-- update users to set ws_id to 0
UPDATE users
SET ws_id = 0 WHERE id = 0;

-- alter table users to make ws_id not nullable
ALTER TABLE users
ALTER COLUMN ws_id SET NOT NULL;
