CREATE TYPE CONVERSATION_TYPE AS ENUM ('PRIVATE', 'GROUP');

CREATE TABLE "conversation"
(
    id         BIGSERIAL PRIMARY KEY,
    private_id VARCHAR(255)      NULL UNIQUE,
    author_id  BIGINT REFERENCES "user" (id),
    type       CONVERSATION_TYPE NOT NULL,
    name       VARCHAR(255)      NULL,
    photo_url  VARCHAR(512)      NULL,
    deleted_at timestamptz       null,
    created_at TIMESTAMPTZ       NOT NULL,
    updated_at TIMESTAMPTZ       NOT NULL
);