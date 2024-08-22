-- Add up migration script here

create table users (
    username text primary key not null,
    pass_hash text not null,
    pass_salt text not null
);
