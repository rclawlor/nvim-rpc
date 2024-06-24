/// Error types for genapi
#[derive(Debug)]
pub enum Error {
    /// An error when saving the generated API
    IoError(String),
    /// An error when rendering the templates
    RenderError(String),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<handlebars::RenderError> for Error {
    fn from(value: handlebars::RenderError) -> Self {
        Self::RenderError(value.to_string())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Error::IoError(err) => err,
                Error::RenderError(err) => err,
            }
        )
    }
}
