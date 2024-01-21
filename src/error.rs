use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Request failed")]
    RequestError(),

    #[error("Could not read cache")]
    CacheError(#[from] std::io::Error),

    #[error("Could not deserialize")]
    Deserialize(#[from] serde_json::Error),

    #[error("Request succeded but the material list is empty")]
    EmptyArray(),

    #[error("Wrong GWP format")]
    GwpError,

    #[error("Deserialization error")]
    DeserializationError(serde_json::Error),

    #[error("Serialization error")]
    SerializationError,

    #[error("Wrong Unit format")]
    UnitError,
    #[error("Api rejected authentication")]
    AuthError,
}
