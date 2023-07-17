create table user_roles (
    id uuid default gen_random_uuid() primary key,
    name varchar(50) not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create trigger update_updated_at_trigger
    before update on user_roles
    for each row
    execute function update_updated_at();

create table users (
    id uuid default gen_random_uuid() primary key ,
    name varchar(200) not null,
    role_id uuid references user_roles (id) not null,
    email varchar(255) unique not null,
    last_token text unique default null, 
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create trigger update_updated_at_trigger
    before update on users
    for each row
    execute function update_updated_at();

create table projects (
    id uuid default gen_random_uuid() primary key,
    name varchar(200) not null,
    repository_url text not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create trigger update_updated_at_trigger
    before update on projects
    for each row
    execute function update_updated_at();

create table users_projects (
    user_id uuid references users (id) not null,
    project_id uuid references projects (id) not null,
    primary key (user_id, project_id),
    is_active boolean default true not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create trigger update_updated_at_trigger
    before update on users_projects
    for each row
    execute function update_updated_at();