use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use crate::traits::TaskProcessResult;

#[derive(Debug)]
pub enum ManagerErrorKind {
    NetworkError,        // No internet
    UnreachableError,    // No response
    InvalidTask,         // Task is not correct
}

#[derive(Debug)]
pub enum DownloadErrorKind {
    NetworkError,        // No internet
    UnreachableError,    // No response
    InvalidURL,          // URL is invalid
    InvalidPage,         // Could not make sense of downloaded material
}

#[derive(Debug)]
pub enum ExtractErrorKind {
    ParsingError,        // Page could not be parsed
}

#[derive(Debug)]
pub enum NormaliseErrorKind {
    ParsingError,        // Could not parse url
}

#[derive(Debug)]
pub enum ArchiveErrorKind {
    NetworkError,        // No internet
    UnreachableError,    // No response
    ServerError,         // Backend received data, but was unable to process it
    InvalidData,         // Data is invalid
}

/// Base error for the web scraper components
#[derive(Debug)]
pub struct ScraperError<K>
where
    K: Display + Debug
{
    pub kind: K,
    msg: String,
    source: Option<Box<dyn Error>>,
}

// Specific error categories
pub type ManagerError = ScraperError<ManagerErrorKind>;
pub type DownloadError = ScraperError<DownloadErrorKind>;
pub type ExtractError = ScraperError<ExtractErrorKind>;
pub type NormaliseError = ScraperError<NormaliseErrorKind>;
pub type ArchiveError = ScraperError<ArchiveErrorKind>;

// std Results with web scraper errors
pub type ManagerResult<T> = std::result::Result<T, ManagerError>;
pub type DownloadResult<T> = std::result::Result<T, DownloadError>;
pub type ExtractResult<T> = std::result::Result<T, ExtractError>;
pub type NormaliseResult<T> = std::result::Result<T, NormaliseError>;
pub type ArchiveResult<T> = std::result::Result<T, ArchiveError>;


// Allows our errors to have source errors or causes like rust's builtin errors.
// The source error is a field in the struct and is optional.
impl<K> Error for ScraperError<K>
where
    K: Display + Debug
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|err| err.deref())
    }
}


impl ManagerError {
    /// Create a new ManagerError with a kind, message, and optional source error.
    pub fn new(kind: ManagerErrorKind, msg: &str, source: Option<Box<dyn Error>>) -> Self {
        ManagerError {
            kind,
            msg: String::from(msg),
            source,
        }
    }
}

// Allows our error to be converted to a appropriate TaskProcessResult
impl From<ManagerError> for TaskProcessResult {
    fn from(e: ScraperError<ManagerErrorKind>) -> Self {
        match e.kind {
            ManagerErrorKind::NetworkError => TaskProcessResult::Reject,
            ManagerErrorKind::UnreachableError => TaskProcessResult::Reject,
            ManagerErrorKind::InvalidTask => TaskProcessResult::Err,
        }
    }
}

impl DownloadError {
    /// Create a new DownloadError with a kind, message, and optional source error.
    pub fn new(kind: DownloadErrorKind, msg: &str, source: Option<Box<dyn Error>>) -> Self {
        DownloadError {
            kind,
            msg: String::from(msg),
            source,
        }
    }
}

// Allows our error to be converted to a appropriate TaskProcessResult
impl From<DownloadError> for TaskProcessResult {
    fn from(e: ScraperError<DownloadErrorKind>) -> Self {
        match e.kind {
            DownloadErrorKind::NetworkError => TaskProcessResult::Reject,
            DownloadErrorKind::UnreachableError => TaskProcessResult::Reject,
            DownloadErrorKind::InvalidURL => TaskProcessResult::Err,
            DownloadErrorKind::InvalidPage => TaskProcessResult::Err,
        }
    }
}

impl ExtractError {
    /// Create a new ExtractError with a kind, message, and optional source error.
    pub fn new(kind: ExtractErrorKind, msg: &str, source: Option<Box<dyn Error>>) -> Self {
        ExtractError {
            kind,
            msg: String::from(msg),
            source,
        }
    }
}

// Allows our error to be converted to a appropriate TaskProcessResult
impl From<ExtractError> for TaskProcessResult {
    fn from(e: ScraperError<ExtractErrorKind>) -> Self {
        match e.kind {
            ExtractErrorKind::ParsingError => TaskProcessResult::Err,
        }
    }
}

impl NormaliseError {
    /// Create a new NormaliseError with a kind, message, and optional source error.
    pub fn new(kind: NormaliseErrorKind, msg: &str, source: Option<Box<dyn Error>>) -> Self {
        NormaliseError {
            kind,
            msg: String::from(msg),
            source,
        }
    }
}

// Allows our error to be converted to a appropriate TaskProcessResult
impl From<NormaliseError> for TaskProcessResult {
    fn from(e: ScraperError<NormaliseErrorKind>) -> Self {
        match e.kind {
            NormaliseErrorKind::ParsingError => TaskProcessResult::Err,
        }
    }
}

impl ArchiveError {
    /// Create a new ArchiveError with a kind, message, and optional source error.
    pub fn new(kind: ArchiveErrorKind, msg: &str, source: Option<Box<dyn Error>>) -> Self {
        ArchiveError {
            kind,
            msg: String::from(msg),
            source,
        }
    }
}

// Allows our error to be converted to a appropriate TaskProcessResult
impl From<ArchiveError> for TaskProcessResult {
    fn from(e: ScraperError<ArchiveErrorKind>) -> Self {
        match e.kind {
            ArchiveErrorKind::NetworkError => TaskProcessResult::Reject,
            ArchiveErrorKind::UnreachableError => TaskProcessResult::Reject,
            ArchiveErrorKind::ServerError => TaskProcessResult::Reject,
            ArchiveErrorKind::InvalidData => TaskProcessResult::Err,
        }
    }
}


// Allows our errors to be displayed
impl<K> Display for ScraperError<K>
where
    K: Display + Debug
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Formats the error as follows:
        // in general:      "<Kind>: <msg>[ (source: <source error>)]"
        // Examples:
        // without source:  "NetworkError: No internet"
        // with source:     "ParsingError: Failed to parse the given data (source: No header file)"
        let source_str = if let Some(s) = &self.source {
            format!(" (source: {})", s)
        } else {
            String::from("")
        };
        write!(f, "{}: {}{}", self.kind, self.msg, source_str)
    }
}

impl Display for ManagerErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for DownloadErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for ExtractErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for NormaliseErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Display for ArchiveErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[cfg(test)]
mod tests {
    use crate::errors::{ArchiveError, ArchiveErrorKind, DownloadError, DownloadErrorKind, ExtractError, ExtractErrorKind, ManagerError, ManagerErrorKind, NormaliseError, NormaliseErrorKind};

    /// Testing formatting of ManagerError without source error
    #[test]
    fn display_manager_error_no_source() {
        let error = ManagerError::new(
            ManagerErrorKind::NetworkError,
            "Some message",
            None,
        );
        let expected_str = "NetworkError: Some message";
        assert_eq!(format!("{}", error), expected_str);
    }

    /// Testing formatting of DownloadError without source error
    #[test]
    fn display_download_error_no_source() {
        let error = DownloadError::new(
            DownloadErrorKind::InvalidURL,
            "URL was an empty string",
            None,
        );
        let expected_str = "InvalidURL: URL was an empty string";
        assert_eq!(format!("{}", error), expected_str);
    }

    /// Testing formatting of ExtractError with source error
    #[test]
    fn display_extract_error_with_source() {
        let error = ExtractError::new(
            ExtractErrorKind::ParsingError,
            "Could not parse data",
            Some(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Data was not found"))),
        );
        let expected_str = "ParsingError: Could not parse data (source: Data was not found)";
        assert_eq!(format!("{}", error), expected_str);
    }

    /// Testing formatting of ArchiveError with source error
    #[test]
    fn display_archive_error_with_source() {
        let error = ArchiveError::new(
            ArchiveErrorKind::ServerError,
            "Server tried to download something and failed",
            Some(Box::new(NormaliseError::new(
                NormaliseErrorKind::ParsingError,
                "Trying to test nested errors",
                None,
            ))),
        );
        let expected_str = "ServerError: Server tried to download something and failed (source: ParsingError: Trying to test nested errors)";
        assert_eq!(format!("{}", error), expected_str);
    }
}