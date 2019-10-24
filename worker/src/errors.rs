use std::error::Error;
use std::fmt::{Display, Debug, Formatter};
use std::ops::Deref;

#[derive(Debug)]
pub enum ManagerErrorKind {
    NetworkError,        // No internet
    UnreachableError,    // No responds
    InvalidTask,         // Task is not correct
}

#[derive(Debug)]
pub enum DownloadErrorKind {
    NetworkError,        // No internet
    UnreachableError,    // No responds
    InvalidURL,          // URL is invalid
}

#[derive(Debug)]
pub enum ExtractErrorKind {
    ParsingError,        // Page could not be parsed
}

#[derive(Debug)]
pub enum ArchiveErrorKind {
    NetworkError,        // No internet
    UnreachableError,    // No responds
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
pub type ArchiveError = ScraperError<ArchiveErrorKind>;

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

impl Display for ArchiveErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[cfg(test)]
mod tests {
    use crate::errors::{ManagerError, ManagerErrorKind, DownloadError, DownloadErrorKind, ExtractErrorKind, ExtractError, ArchiveError, ArchiveErrorKind};
    use std::fmt::Display;

    /// Testing formatting of ManagerError without source error
    #[test]
    fn display_manager_error_no_source() {
        let error = ManagerError {
            kind: ManagerErrorKind::NetworkError,
            msg: String::from("Some message"),
            source: None,
        };
        let expected_str = "NetworkError: Some message";
        assert_eq!(format!("{}", error), expected_str);
    }

    /// Testing formatting of DownloadError without source error
    #[test]
    fn display_download_error_no_source() {
        let error = DownloadError {
            kind: DownloadErrorKind::InvalidURL,
            msg: String::from("URL was an empty string"),
            source: None,
        };
        let expected_str = "InvalidURL: URL was an empty string";
        assert_eq!(format!("{}", error), expected_str);
    }

    /// Testing formatting of ExtractError with source error
    #[test]
    fn display_extract_error_with_source() {
        let error = ExtractError {
            kind: ExtractErrorKind::ParsingError,
            msg: String::from("Could not parse data"),
            source: Some(Box::new(std::io::Error::new(std::io::ErrorKind::NotFound, "Data was not found"))),
        };
        let expected_str = "ParsingError: Could not parse data (source: Data was not found)";
        assert_eq!(format!("{}", error), expected_str);
    }

    /// Testing formatting of ArchiveError with source error
    #[test]
    fn display_archive_error_with_source() {
        let error = ArchiveError {
            kind: ArchiveErrorKind::ServerError,
            msg: String::from("Server tried to download something and failed"),
            source: Some(Box::new(DownloadError {
                kind: DownloadErrorKind::NetworkError,
                msg: String::from("Trying to test nested errors"),
                source: None,
            })),
        };
        let expected_str = "ServerError: Server tried to download something and failed (source: NetworkError: Trying to test nested errors)";
        assert_eq!(format!("{}", error), expected_str);
    }
}