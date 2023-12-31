-- create table user_roles (
--     id uuid default gen_random_uuid() primary key,
--     name varchar(50) not null,
--     created_at timestamp default now() not null,
--     updated_at timestamp default now() not null
-- );

-- create trigger update_updated_at_trigger before
-- update
--     on user_roles for each row execute function update_updated_at();

create table users (
    id uuid default gen_random_uuid() primary key,
    name varchar(200) not null,
    -- role_id uuid references user_roles (id) not null,
    email varchar(255) unique not null,
    access_token text unique default null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create trigger update_updated_at_trigger before
update
    on users for each row execute function update_updated_at();

create table user_refresh_tokens (
    user_id uuid references users (id) not null,
    refresh_token_cypher bytea not null,
    cypher_nonce bytea not null,
    refresh_token_expires_in integer not null,
    scope varchar(20) not null,
    token_type varchar(20) not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null,
    primary key (user_id)
);

create trigger update_updated_at_trigger before
update
    on user_refresh_tokens for each row execute function update_updated_at();

create table repositories (
    id uuid default gen_random_uuid() primary key,
    name varchar(50) not null,
    owner varchar(50) not null,
    is_organization boolean not null,
    design_file_sha varchar(50),
    html_url text not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null,
    unique (name, owner, is_organization)
);

create trigger update_updated_at_trigger before
update
    on repositories for each row execute function update_updated_at();

create table designs (
    id uuid default gen_random_uuid() primary key,
    data jsonb default '{}' not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create trigger update_updated_at_trigger before
update
    on designs for each row execute function update_updated_at();

create table projects (
    id uuid default gen_random_uuid() primary key,
    name varchar(200) not null,
    repo_id uuid references repositories (id),
    design_id uuid references designs (id) not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create trigger update_updated_at_trigger before
update
    on projects for each row execute function update_updated_at();

create table users_projects (
    user_id uuid references users (id) not null,
    project_id uuid references projects (id) not null,
    primary key (user_id, project_id),
    is_active boolean default true not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create trigger update_updated_at_trigger before
update
    on users_projects for each row execute function update_updated_at();