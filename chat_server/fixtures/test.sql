-- insert workspace
-- workspace table
-- insert 3 workspaces with owner_id 0
INSERT INTO workspaces (name, owner_id)
VALUES ('workspace1', 0),('workspace2', 0),('workspace3', 0);






-- insert 4 chats
-- insert public/private channel
-- chat table
-- insert 2 public channels with name channel1, channel2 and members 1,2,3,4,5
INSERT INTO chats (name, type, members, ws_id)
VALUES ('general', 'public_channel', '{1,2,3,4,5}', 1),
('private', 'private_channel', '{1,2,3}', 1);

-- insert unnamed chat
INSERT INTO chats (type, members, ws_id)
VALUES ('group', '{1,3,4}', 1);

-- message table
-- insert 10 messages
-- insert 10 messages for chat 1 with sender 1
INSERT INTO messages (chat_id, sender_id, content)
VALUES (1, 1, 'message1'),
(1, 1, 'message2'),
(1, 1, 'message3'),
(1, 1, 'message4'),
(1, 1, 'message5'),
(1, 2, 'message6'),
(1, 2, 'message7'),
(1, 3, 'message8'),
(1, 4, 'message9'),
(1, 5, 'message10');
