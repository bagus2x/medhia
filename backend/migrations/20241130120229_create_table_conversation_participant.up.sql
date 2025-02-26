CREATE TABLE "conversation_participant"
(
    id              BIGSERIAL PRIMARY KEY,
    conversation_id BIGINT REFERENCES "conversation" (id) NOT NULL,
    user_id         BIGINT REFERENCES "user" (id)         NOT NULL,
    joined_at       TIMESTAMPTZ                           NOT NULL,
    roles           VARCHAR(255)                          NOT NULL,
    deleted_at      timestamptz                           null,
    created_at      TIMESTAMPTZ                           NOT NULL,
    UNIQUE (conversation_id, user_id)
);

CREATE INDEX idx_conversation_participant_user_id ON "conversation_participant" (user_id);
