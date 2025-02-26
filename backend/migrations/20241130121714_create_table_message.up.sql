-- Add up migration script here

CREATE TABLE "message"
(
    id              BIGSERIAL PRIMARY KEY,
    conversation_id BIGINT REFERENCES "conversation" (id) NOT NULL,
    sender_id       BIGINT REFERENCES "user" (id)         not null,
    text            TEXT                                  not null,
    deleted_at      timestamptz                           null,
    created_at      timestamptz                           not null,
    updated_at      timestamptz                           not null
);

CREATE INDEX idx_message_conversation_id ON "message" (conversation_id);