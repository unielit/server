use crate::errors::AppError;
use crate::models::Result;
use crate::schema::*;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Insertable, Queryable, Selectable, Identifiable, Serialize, ToSchema, Debug, PartialEq)]
#[diesel(table_name = designs)]
#[serde(rename_all = "camelCase")]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Design {
    pub id: Uuid,
    pub data: serde_json::Value,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub fn create_design(conn: &mut PgConnection) -> Result<Design> {
    use crate::schema::designs::dsl::*;

    diesel::insert_into(designs)
        .default_values()
        .returning(Design::as_returning())
        .get_result::<Design>(conn)
        .map_err(AppError::from)
}

pub fn get_design(conn: &mut PgConnection, design_id: Uuid) -> Result<Design> {
    use crate::schema::designs::dsl::*;

    designs
        .find(design_id)
        .select(Design::as_select())
        .first(conn)
        .map_err(AppError::from)
}

pub fn update_design(
    conn: &mut PgConnection,
    design_id: Uuid,
    design_data: serde_json::Value,
) -> Result<Design> {
    use crate::schema::designs::dsl::*;

    diesel::update(designs)
        .filter(id.eq(design_id))
        .set(data.eq(design_data))
        .returning(Design::as_returning())
        .get_result(conn)
        .map_err(AppError::from)
}
