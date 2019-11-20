use crate::traits::Normaliser;

use url::Url;
use url_normalizer;
use crate::errors::{NormaliseResult, NormaliseError};
use crate::errors::NormaliseErrorKind::ParsingError;
use std::collections::HashSet;


/// The DefaultNormaliser is a Normaliser that normalises URI's as described by
/// RFC 3986 (https://tools.ietf.org/html/rfc3986) without changing the semantics.
pub struct DefaultNormaliser;

impl Normaliser for DefaultNormaliser {
    /// Normalising the tasks Url by setting scheme and path to lowercase,
    /// removing the dot in path, removes hash from url and ordering the query.
    fn normalise(&self, urls: Vec<Url>) -> Vec<Url> {
        let mut new_urls = urls;
        // Normalise extracted links
        // After normalisation, squash urls into a hash set to remove duplicates
        // Erroneous urls are discarded
        let mut set: HashSet<Url> = new_urls.drain(..)
            .filter_map(|url| {
                let url_as_str = String::from(url.as_str());
                match DefaultNormaliser::full_normalisation(url) {
                    Ok(normalised_url) => Some(normalised_url),
                    Err(e) => {
                        error!("Failed to normalise {}. {}", url_as_str, e);
                        None
                    }
                }
            }).collect();

        set.drain().collect()
    }
}

impl DefaultNormaliser {

    /// Perform all the implemented normalisation functions
    fn full_normalisation(url: Url) -> NormaliseResult<Url> {
        let mut new_url = url;

        // Normalising by ordering the query in alphabetic order,
        // removes hash from url and changes encrypted to unencrypted.
        new_url = url_normalizer::normalize(new_url).map_err(|_| {
            NormaliseError::new(ParsingError, "Failed to normalise using url library", None)
        })?;

        // Converting encoded triplets to uppercase
        new_url = DefaultNormaliser::converting_encoded_triplets_to_upper_for_url(new_url)?;

        // Sets the scheme and host to lowercase
        new_url = DefaultNormaliser::scheme_and_host_to_lowercase(new_url)?;

        Ok(new_url)
    }

    //The parser operation given by the Url-crate features an automatically normalisation of the given string,
    //because that is the case, there are no need for the below functions.
    /// Sets the scheme and host to lowercase
    fn scheme_and_host_to_lowercase(url: Url) -> NormaliseResult<Url> {
        let mut new_url = url;

        if let Some(host) = new_url.host_str() {
            let host = host.to_lowercase();
            new_url.set_host(Some(host.as_str())).map_err(|e| {
                NormaliseError::new(ParsingError, "Failed to make host lower-case", Some(Box::new(e)))
            })?;
        }

        let scheme = new_url.scheme().to_lowercase();
        new_url.set_scheme(scheme.as_str()).map_err(|_| {
            NormaliseError::new(ParsingError, "Failed to make scheme lower-case", None)
        })?;

        Ok(new_url)
    }

    /// Converting encoded triplets to uppercase, example:
    /// From: "http://example.com/foo%2a"
    /// To: "http://example.com/foo%2A"
    fn converting_encoded_triplets_to_upper_for_url(url: Url) -> NormaliseResult<Url> {
        let mut new_url = url;

        let mut path = DefaultNormaliser::converting_encoded_triplet_to_upper_for_str(new_url.path());
        new_url.set_path(path.as_str());

        if new_url.query().is_some() {
            let mut query = DefaultNormaliser::converting_encoded_triplet_to_upper_for_str(new_url.query().unwrap());
            new_url.set_query(Option::Some(query.as_str()));
        }

        if new_url.fragment().is_some() {
            let mut fragment = DefaultNormaliser::converting_encoded_triplet_to_upper_for_str(new_url.fragment().unwrap());
            new_url.set_fragment(Option::Some(fragment.as_str()));
        }
        Ok(new_url)
    }

    /// This function look for "%" and then convert the
    /// following two letters to uppercase for a given string.
    fn converting_encoded_triplet_to_upper_for_str(some_str: &str) -> String {
        let mut str_build = "".to_string();
        let some_chars = some_str.chars();
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

        str_build
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;

    #[test]
    fn test_empty_path_to_slash() {
        let expected_url = "http://example.com/";
        let test_task = Task {
            url: Url::parse("http://example.com").unwrap()
        };

        let test_vec = vec![test_task.url];

        let test_url = DefaultNormaliser.normalise(test_vec);

        assert_eq!(test_url[0].to_string(), expected_url);
    }

    #[test]
    fn test_converting_encoded_triplets_to_upper() {
        let expected_url = "http://example.com/foo%2A";
        let test_task = Task {
            url: Url::parse("http://example.com/foo%2a").unwrap()
        };

        let test_url = DefaultNormaliser::converting_encoded_triplets_to_upper_for_url(test_task.url).unwrap();

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

        assert_eq!("https", test_url.scheme());
    }

    #[test]
    fn test_scheme_and_host_to_lowercase2() {
        let test_task = Task {
            url: Url::parse("HTTPS://user:pass@sub.HOST.cOm:8080/p/a/t/h?query=string#hash")
                .unwrap(),
        };

        let test_url = DefaultNormaliser::scheme_and_host_to_lowercase(test_task.url).unwrap();

        assert_eq!("sub.host.com", test_url.host_str().unwrap());
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
