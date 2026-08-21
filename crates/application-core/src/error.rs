#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Database error: {message}")]
    Sqlite {
        message: String,
        category: Option<vrcx_0_persistence::SqliteErrorCategory>,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Update artifact is invalid: {0}")]
    UpdateArtifactInvalid(String),

    #[error("{0}")]
    PersistenceInvalidData(String),

    #[error("{0}")]
    RegistryPolicyInvalid(String),

    #[error("{0}")]
    WebClient(String),

    #[error("{message}")]
    VrchatApi { status_code: i32, message: String },

    #[error("{0}")]
    Custom(String),
}

impl From<vrcx_0_core::vrchat_registry_policy::RegistryPolicyError> for Error {
    fn from(value: vrcx_0_core::vrchat_registry_policy::RegistryPolicyError) -> Self {
        use vrcx_0_core::vrchat_registry_policy::RegistryPolicyError;
        match value {
            RegistryPolicyError::Json(error) => Self::Json(error),
            RegistryPolicyError::Invalid(message) => Self::RegistryPolicyInvalid(message),
        }
    }
}

impl From<vrcx_0_persistence::Error> for Error {
    fn from(value: vrcx_0_persistence::Error) -> Self {
        match value {
            vrcx_0_persistence::Error::Database(message) => Self::Database(message),
            vrcx_0_persistence::Error::Sqlite { message, category } => {
                Self::Sqlite { message, category }
            }
            vrcx_0_persistence::Error::Io(error) => Self::Io(error),
            vrcx_0_persistence::Error::Json(error) => Self::Json(error),
            vrcx_0_persistence::Error::InvalidData(message) => {
                Self::PersistenceInvalidData(message)
            }
            vrcx_0_persistence::Error::Custom(message) => Self::Custom(message),
        }
    }
}

impl From<vrcx_0_media::Error> for Error {
    fn from(value: vrcx_0_media::Error) -> Self {
        match value {
            vrcx_0_media::Error::Io(error) => Self::Io(error),
            vrcx_0_media::Error::Custom(message) => Self::Custom(message),
        }
    }
}

impl From<vrcx_0_vrchat_client::WebClientError> for Error {
    fn from(value: vrcx_0_vrchat_client::WebClientError) -> Self {
        match value {
            vrcx_0_vrchat_client::WebClientError::Custom(message) => Self::WebClient(message),
            vrcx_0_vrchat_client::WebClientError::Io(error) => Self::Io(error),
        }
    }
}

impl From<vrcx_0_vrchat_client::ImageFetchError> for Error {
    fn from(value: vrcx_0_vrchat_client::ImageFetchError) -> Self {
        match value {
            vrcx_0_vrchat_client::ImageFetchError::Custom(message) => Self::Custom(message),
        }
    }
}

impl From<vrcx_0_vrchat_client::HttpApiError> for Error {
    fn from(value: vrcx_0_vrchat_client::HttpApiError) -> Self {
        match value {
            vrcx_0_vrchat_client::HttpApiError::Custom(message) => Self::Custom(message),
        }
    }
}

impl From<vrcx_0_vrchat_client::http_api::VrchatApiFailure> for Error {
    fn from(value: vrcx_0_vrchat_client::http_api::VrchatApiFailure) -> Self {
        Self::VrchatApi {
            status_code: value.status_code,
            message: value.message,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_stable_diagnostic_categories_from_lower_layers() {
        assert!(matches!(
            Error::from(vrcx_0_persistence::Error::InvalidData(
                "invalid snapshot".into()
            )),
            Error::PersistenceInvalidData(message) if message == "invalid snapshot"
        ));
        assert!(matches!(
            Error::from(
                vrcx_0_core::vrchat_registry_policy::RegistryPolicyError::Invalid(
                    "invalid registry value".into()
                )
            ),
            Error::RegistryPolicyInvalid(message) if message == "invalid registry value"
        ));
        assert!(matches!(
            Error::from(vrcx_0_vrchat_client::WebClientError::Custom(
                "request setup failed".into()
            )),
            Error::WebClient(message) if message == "request setup failed"
        ));
    }
}
