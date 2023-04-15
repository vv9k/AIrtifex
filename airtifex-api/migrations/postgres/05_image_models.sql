CREATE TABLE image_models (
     model_id UUID PRIMARY KEY NOT NULL,
     name VARCHAR NOT NULL UNIQUE,
     description VARCHAR
);
