use crate::errors::AppError;

type Result<T> = std::result::Result<T, AppError>;

pub(super) mod users;
pub(super) mod projects;