use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::PgConnection;
use thiserror::Error;

mod __generated_schema;
use __generated_schema::signed_data::{self, dsl::signed_data as signedDataTable};

#[derive(Queryable, Serialize, Deserialize)]
pub struct SignedData {
    pub id: i64,
    pub created_at: NaiveDateTime, // Local::now().naive_local()
    //
    pub data_hash_b64: String,
}

#[derive(Insertable)]
#[table_name = "signed_data"]
pub struct NewSignedData<'a> {
    pub data_hash_b64: &'a str,
    pub created_at: Option<NaiveDateTime>,
}
impl<'a> NewSignedData<'a> {
    pub fn insert(self, db_conn: &PgConnection) -> Result<SignedData, ModelErr> {
        let user: SignedData = diesel::insert_into(signedDataTable)
            .values(&self)
            .get_result(db_conn)?;
        Ok(user.into())
    }
}

#[derive(Error, Debug)]
pub enum ModelErr {
    #[error("already exists: {0}")]
    AlreadyExists(diesel::result::Error),
    #[error(transparent)]
    OtherDieselErr(diesel::result::Error),
}
impl From<diesel::result::Error> for ModelErr {
    fn from(e: diesel::result::Error) -> Self {
        use diesel::result::{DatabaseErrorKind, Error as DieselErr};
        match &e {
            DieselErr::DatabaseError(kind, _info) => match kind {
                DatabaseErrorKind::UniqueViolation => ModelErr::AlreadyExists(e),
                _ => ModelErr::OtherDieselErr(e),
            },
            _ => ModelErr::OtherDieselErr(e),
        }
    }
}
