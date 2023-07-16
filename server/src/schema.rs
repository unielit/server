// @generated automatically by Diesel CLI.

diesel::table! {
    projects (id) {
        id -> Uuid,
        #[max_length = 200]
        name -> Varchar,
        repository_url -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    user_roles (id) {
        id -> Uuid,
        #[max_length = 50]
        name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 200]
        name -> Varchar,
        role_id -> Uuid,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 100]
        last_token -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users_projects (user_id, project_id) {
        user_id -> Uuid,
        project_id -> Uuid,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(users -> user_roles (role_id));
diesel::joinable!(users_projects -> projects (project_id));
diesel::joinable!(users_projects -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    projects,
    user_roles,
    users,
    users_projects,
);
