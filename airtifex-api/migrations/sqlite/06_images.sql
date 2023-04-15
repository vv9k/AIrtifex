CREATE TABLE images (
     id UUID        PRIMARY KEY NOT NULL,
     user_id        UUID NOT NULL,
     width          INTEGER NOT NULL,
     height         INTEGER NOT NULL,
     model          VARCHAR NOT NULL references image_models(name),
     prompt         VARCHAR NOT NULL,
     n_steps        INTEGER NOT NULL,
     seed           INTEGER NOT NULL,
     num_samples    INTEGER NOT NULL,
     guidance_scale REAL NOT NULL,
     processing     BOOLEAN NOT NULL,
     create_date    DATETIME NOT NULL,

     CONSTRAINT fk_user
       FOREIGN KEY (user_id)
       REFERENCES users (id)
       ON DELETE CASCADE
);
