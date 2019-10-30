use crate::task::Task;
use crate::traits::Normaliser;

use std::error::Error;
use std::fmt;
use url::Url;
use url_normalizer;
use rand::seq::index::sample;
use crate::errors::{NormaliseResult, NormaliseError, NormaliseErrorKind};

pub struct DefaultNormaliser;

impl Normaliser for DefaultNormaliser {
    /// Normalising the tasks URL by setting scheme and path to lowercase,
    /// removing the dot in path, removes hash from url and ordering the query.
    fn normalise(&self, url: Url) -> NormaliseResult<Url> {
        DefaultNormaliser::full_normalisation(url)
    }
}

impl DefaultNormaliser {
    /// Run through all implemented normalisation functions and applying those on the given url.
    fn full_normalisation(url: Url) -> NormaliseResult<Url> {
        let mut new_url = url;

        //Normalising by ordering the query in alphabetic order,
        //removes hash from url and changes encrypted to unencrypted.
        new_url = url_normalizer::normalize(new_url).map_err(|_| {
            NormaliseError::new(NormaliseErrorKind::ParsingError, String::from("Failed to normalise using url library"), None)
        })?;

        new_url = DefaultNormaliser::scheme_and_host_to_lowercase(new_url)?;
        new_url = DefaultNormaliser::converting_encoded_triplets_to_upper(new_url)?;
        new_url = DefaultNormaliser::empty_path_to_slash(new_url)?;

        Ok(new_url)
    }

    ///Sets the scheme and host to lowercase
    fn scheme_and_host_to_lowercase(url: Url) -> NormaliseResult<Url> {
        let mut new_url = url;

        if let Some(host) = new_url.host_str() {
            let host = host.to_lowercase();
            new_url.set_host(Some(host.as_str())).map_err(|e| {
                NormaliseError::new(NormaliseErrorKind::ParsingError, String::from("Failed to make host lower-case"), Some(Box::new(e)))
            })?;
        }

        let scheme = new_url.scheme().to_lowercase();
        new_url.set_scheme(scheme.as_str()).map_err(|_| {
            NormaliseError::new(NormaliseErrorKind::ParsingError, String::from("Failed to make scheme lower-case"), None)
        })?;

        Ok(new_url)
    }

    /// Converting encoded triplets to uppercase, example:
    /// From: "http://example.com/foo%2a"
    /// To: "http://example.com/foo%2A"
    fn converting_encoded_triplets_to_upper(url: Url) -> NormaliseResult<Url> {
        let new_url = url;
        let mut str_build = "".to_string();
        let some_chars = new_url.as_str().chars();
        let mut counter = 0;

        // Iterating through all characters in the url
        // and building a new string for creating a new url
        for symbol in some_chars {
            // If the symbol "%" is read, the next two symbols
            // will be converted to uppercase and added to the builder otherwise just
            // add the symbol to the builder.
            let symbol_as_str = symbol.to_string();
            if symbol_as_str == "%" {
                counter = 3;
            }
            if counter > 0 {
                str_build.push_str(symbol_as_str.to_uppercase().as_str());
                counter = counter - 1;
            } else {
                str_build.push_str(symbol_as_str.as_str());
            }
        }

        // Parse the built string as an url for return
        Url::parse(str_build.as_str()).map_err(|e| {
            NormaliseError::new(NormaliseErrorKind::ParsingError, String::from("Failed converting triplets to uppercase"), Some(Box::new(e)))
        })
    }

    /// Converting an empty path to a slash. Example:
    /// From: "http://example.com"
    /// To: "http://example.com/"
    fn empty_path_to_slash(url: Url) -> NormaliseResult<Url> {
        let new_url = url;
        let url_as_str = new_url.as_str();

        if url_as_str.ends_with("/") || !new_url.path().is_empty() {
            Ok(new_url)
        } else {
            url_as_str.to_string().push_str("/");
            Url::parse(url_as_str).map_err(|e| {
                NormaliseError::new(NormaliseErrorKind::ParsingError, String::from("Failed adding '/' for empty path"), Some(Box::new(e)))
            })
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_path_to_slash() {
        let expected_url = "http://example.com/";
        let mut test_task = Task {
            url: Url::parse("http://example.com").unwrap()
        };

        let test_url = DefaultNormaliser::empty_path_to_slash(test_task.url).unwrap();

        assert_eq!(test_url.to_string(), expected_url);
    }

    #[test]
    fn test_converting_encoded_triplets_to_upper() {
        let expected_url = "http://example.com/foo%2A";
        let mut test_task = Task {
            url: Url::parse("http://example.com/foo%2a").unwrap()
        };

        let test_url = DefaultNormaliser::converting_encoded_triplets_to_upper(test_task.url).unwrap();

        assert_eq!(test_url.to_string(), expected_url);
    }

    #[test]
    fn test_scheme_and_host_to_lowercase0() {
        let expected_url = "https://user:pass@sub.host.com:8080/p/a/t/h?query=string#hash";

        let test_task = Task {
            url: Url::parse("HTTPS://user:pass@sub.HOST.cOm:8080/p/a/t/h?query=string#hash")
                .unwrap(),
        };

        let test_url = DefaultNormaliser::scheme_and_host_to_lowercase(test_task.url).unwrap();

        assert_eq!(test_url.to_string(), expected_url);
    }

    #[test]
    fn test_scheme_and_host_to_lowercase1() {
        let test_task = Task {
            url: Url::parse("HTTPS://user:pass@sub.HOST.cOm:8080/p/a/t/h?query=string#hash")
                .unwrap(),
        };

        let test_url = DefaultNormaliser::scheme_and_host_to_lowercase(test_task.url).unwrap();

        let test_scheme = test_url.scheme();
        let test_host = test_url.host_str().unwrap();

        let expected_scheme = "https";
        let expected_host = "sub.host.com";

        assert_eq!(test_scheme, expected_scheme);
    }

    #[test]
    fn test_scheme_and_host_to_lowercase2() {
        let test_task = Task {
            url: Url::parse("HTTPS://user:pass@sub.HOST.cOm:8080/p/a/t/h?query=string#hash")
                .unwrap(),
        };

        let test_url = DefaultNormaliser::scheme_and_host_to_lowercase(test_task.url).unwrap();

        let test_scheme = test_url.scheme();
        let test_host = test_url.host_str().unwrap();

        let expected_scheme = "https";
        let expected_host = "sub.host.com";

        assert_eq!(test_host, expected_host);
    }

    #[test]
    fn test_scheme_and_host_to_lowercase3() {
        let test_task = Task {
            url: Url::parse("urn:oasis:names:specification:docbook:dtd:xml:4.1.2")
                .unwrap(),
        };

        let test_url = DefaultNormaliser::scheme_and_host_to_lowercase(test_task.url).unwrap();

        assert_eq!(test_url.has_host(), false)
    }
}
