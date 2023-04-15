CREATE TABLE image_samples (
     sample_id UUID PRIMARY KEY NOT NULL,
     image_id UUID NOT NULL,
     n INTEGER NOT NULL,
     data BLOB NOT NULL,

     CONSTRAINT fk_image
       FOREIGN KEY (image_id)
       REFERENCES images (id)
       ON DELETE CASCADE
);