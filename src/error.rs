use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    LoginFail,

    // -- Auth errors.
    AuthFailNoAuthTokenCookie,
    AuthFailTokenWrongFormat,
    AuthFailCtxNotInRequestExt,
    CustomerError,
    HandlerError,

    // DB Errors
    PGError { e: String },
    SqlxUuid { e: String },
    MongoParsingError { e: String },
    MongoConnectionError { e: String },
    MongoQueryError { e: String },
    MongoInvalidIDError { e: String },
    MongoSerializeBsonError { e: String },

    MongoSerializeError,
    MongoError,

    // -- Model errors.
    TicketDeleteFailIdNotFound { id: u64 },
}

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
// end region: --- Error Boilerplate

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::info!("->> {:<12} - {self:?}", "INTO_RES");

        // Create a placeholder Axum response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response.
        response.extensions_mut().insert(self);

        response
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        #[allow(unreachable_patterns)]
        match self {
            Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            // -- Auth.
            Self::AuthFailNoAuthTokenCookie
            | Self::AuthFailTokenWrongFormat
            | Self::AuthFailCtxNotInRequestExt => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            // -- Model.
            Self::CustomerError => {
                tracing::error!("Customer Error");
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::PGError { e } => {
                tracing::error!("DB Error {}", e);
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::HandlerError => {
                tracing::error!("DB Error");
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::SqlxUuid { e } => {
                tracing::error!("DB Error {}", e);
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::MongoParsingError { e } => {
                tracing::error!("DB Error {}", e);
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::MongoConnectionError { e } => {
                tracing::error!("DB Error {}", e);
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::MongoQueryError { e } => {
                tracing::error!("DB Error {}", e);
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::MongoInvalidIDError { e } => {
                tracing::error!("DB Error {}", e);
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::MongoSerializeBsonError { e } => {
                tracing::error!("DB Error {}", e);
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::MongoSerializeError => {
                tracing::error!("DB Error");
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::MongoError => {
                tracing::error!("DB Error");
                (StatusCode::BAD_REQUEST, ClientError::DATABASE_ERROR)
            }

            // -- Model.
            Self::TicketDeleteFailIdNotFound { .. } => {
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }

            // -- Fallback.
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    INVALID_PARAMS,
    DATABASE_ERROR,
    SERVICE_ERROR,
}
