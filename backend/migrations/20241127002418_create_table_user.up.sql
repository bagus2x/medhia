-- Add up migration script here
create table "user"
(
    id         bigserial,
    username   varchar(255) not null unique,
    email      varchar(255) not null unique,
    password   varchar(255) not null,
    name       varchar(255) not null,
    photo_url  varchar(255) null,
    deleted_at timestamptz  null,
    created_at timestamptz  not null,
    updated_at timestamptz  not null
);