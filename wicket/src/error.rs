/// Errors that can occur during Wikipedia dump extraction.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// XML parse error.
    #[error("XML parse error: {0}")]
    Xml(String),

    /// XML reader error from quick-xml.
    #[error("XML reader error: {0}")]
    XmlReader(#[from] quick_xml::Error),

    /// Invalid file size specification.
    #[error("invalid file size: {0}")]
    InvalidFileSize(String),

    /// JSON serialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type alias for wicket operations.
pub type Result<T> = std::result::Result<T, Error>;
