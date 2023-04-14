CREATE TABLE chats (
     id UUID PRIMARY KEY NOT NULL,
     username VARCHAR NOT NULL references users(username),
     model VARCHAR NOT NULL references llm_models(name),
     title VARCHAR,
     start_date DATETIME,

     num_predict INTEGER,
     system_prompt VARCHAR,
     n_batch INTEGER,
     top_k INTEGER,
     top_p FLOAT,
     repeat_penalty FLOAT,
     temp FLOAT
);
