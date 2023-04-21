CREATE TABLE prompts (
     id UUID PRIMARY KEY NOT NULL,
     username VARCHAR NOT NULL references users(username),
     model VARCHAR NOT NULL references llm_models(name),
     prompt VARCHAR NOT NULL,
     response VARCHAR NOT NULL,
     date DATETIME,

     num_predict INTEGER,
     system_prompt VARCHAR,
     n_batch INTEGER,
     top_k INTEGER,
     top_p FLOAT,
     repeat_penalty FLOAT,
     temp FLOAT
);
