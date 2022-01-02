-- Add migration script here

create table posts (
  id uuid default gen_random_uuid() not null primary key,
  title text not null,
  body text not null,
  created_at timestamp not null default now(),
  updated_at timestamp not null default now()
);
