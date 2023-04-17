CREATE TABLE image_models (
     model_id                UUID PRIMARY KEY NOT NULL,
     name                    VARCHAR NOT NULL UNIQUE,
     description             VARCHAR,
     feature_inpaint         BOOLEAN,
     feature_text_to_image   BOOLEAN,
     feature_image_to_image  BOOLEAN
);
