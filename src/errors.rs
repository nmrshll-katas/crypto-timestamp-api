use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::convert::Infallible;
use warp::filters::body::BodyDeserializeError;
use warp::http::StatusCode;
use warp::{Rejection, Reply};
//
use crate::routes::SignDataErr;

pub async fn handle_rejection(r: Rejection) -> Result<impl Reply, Infallible> {
    Ok(ErrResp::from(r).into_reply())
}

/// An API error serializable to JSON responses
struct ErrResp {
    statuscode: StatusCode,
    message: String,
}
impl ErrResp {
    pub fn new(code: StatusCode, msg: &str) -> Self {
        ErrResp {
            statuscode: code,
            message: msg.into(),
        }
    }
    pub fn into_reply(&self) -> impl warp::Reply {
        warp::reply::with_status(warp::reply::json(&self), self.statuscode)
    }
}
impl Serialize for ErrResp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ErrResp", 3)?;
        state.serialize_field("code", &self.statuscode.as_u16())?;
        state.serialize_field("message", &self.message)?;
        state.serialize_field("status", "error")?;
        state.end()
    }
}
impl From<Rejection> for ErrResp {
    fn from(r: Rejection) -> Self {
        if r.is_not_found() {
            return ErrResp::new(StatusCode::NOT_FOUND, "Not found");
        }
        if let Some(e) = r.find::<SignDataErr>() {
            return ErrResp::from(e);
        }
        if let Some(e) = r.find::<BodyDeserializeError>() {
            return ErrResp::new(
                StatusCode::BAD_REQUEST,
                &format!("Bad Request: {}", e).to_owned(),
            );
        }
        if let Some(_) = r.find::<warp::reject::MethodNotAllowed>() {
            return ErrResp::from(StatusCode::METHOD_NOT_ALLOWED);
        }
        ErrResp::new(StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_REJECTION")
    }
}

impl From<StatusCode> for ErrResp {
    fn from(sc: StatusCode) -> Self {
        match sc {
            StatusCode::UNAUTHORIZED => ErrResp::new(sc, "Unauthorized"),
            StatusCode::METHOD_NOT_ALLOWED => ErrResp::new(sc, "Method not Allowed"),
            _ => ErrResp::new(StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLED_REJECTION"),
        }
    }
}
impl From<&SignDataErr> for ErrResp {
    fn from(e: &SignDataErr) -> Self {
        use crate::models::ModelErr;
        match e {
            SignDataErr::DbConn(_) => {
                ErrResp::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            SignDataErr::Model(model_err) => match model_err {
                ModelErr::AlreadyExists(_) => {
                    ErrResp::new(StatusCode::CONFLICT, "Resource already exists")
                }
                ModelErr::OtherDieselErr(_) => {
                    ErrResp::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                }
            },
            SignDataErr::B64DecodeBody(e) => ErrResp::new(
                StatusCode::BAD_REQUEST,
                &format!("Invalid base64 field: {}", e).to_owned(),
            ),
            SignDataErr::SerializeFieldsSigned(_) => {
                ErrResp::new(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            SignDataErr::PowRejected => ErrResp::new(
                StatusCode::BAD_REQUEST,
                "PoW proof didn't pass verification",
            ),
        }
    }
}
