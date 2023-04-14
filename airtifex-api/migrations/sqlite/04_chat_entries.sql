CREATE TABLE chat_entries (
     entry_id UUID PRIMARY KEY NOT NULL,
     chat_id UUID NOT NULL,
     entry_type INTEGER NOT NULL DEFAULT 1 REFERENCES entry_types(type_id),
     content VARCHAR,
     entry_date DATETIME,
     CONSTRAINT fk_chats
       FOREIGN KEY (chat_id)
       REFERENCES chats (id)
       ON DELETE CASCADE
);

CREATE TABLE entry_types (
    type_id INTEGER PRIMARY KEY NOT NULL,
    type CHAR(3) NOT NULL
);

INSERT INTO entry_types (type_id, type) VALUES (1, 'USR');
INSERT INTO entry_types (type_id, type) VALUES (2, 'BOT');
