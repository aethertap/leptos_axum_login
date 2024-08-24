-- Add up migration script here

create table users (
    id integer primary key not null,
    username text not null,
    pass_hash text not null,
    pass_salt text not null,
    unique(username)
);
