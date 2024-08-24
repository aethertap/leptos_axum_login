-- Add up migration script here

create table Profiles (
    id integer primary key not null,
    user_id integer not null,
    email text,
    photo_url text,
    about text,
    foreign key (user_id) references users(id)
);

