CREATE TABLE users (
     id UUID PRIMARY KEY NOT NULL,
     username VARCHAR NOT NULL UNIQUE,
     email VARCHAR NOT NULL UNIQUE,
     password BLOB NOT NULL,
     account_type INTEGER NOT NULL DEFAULT 2 REFERENCES user_types(type_id),
     registration_date DATETIME
);

CREATE TABLE user_types (
    type_id INTEGER PRIMARY KEY NOT NULL,
    type CHAR(3) NOT NULL
);

INSERT INTO user_types (type_id, type) VALUES (1, 'ADM');
INSERT INTO user_types (type_id, type) VALUES (2, 'USR');
INSERT INTO user_types (type_id, type) VALUES (4, 'SVC');
