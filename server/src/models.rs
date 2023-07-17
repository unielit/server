use crate::errors::AppError;

pub type Result<T> = std::result::Result<T, AppError>;

pub(super) mod users;
pub(super) mod projects;