use crate::task::Task;
use crate::traits::Normaliser;

use std::error::Error;
use std::fmt;
use url::Url;
use url_normalizer;
use rand::seq::index::sample;

pub struct DefaultNormaliser;

impl Normaliser for DefaultNormaliser {
    /// Normalising the tasks URL by setting scheme and path to lowercase,
    /// removing the dot in path, removes hash from url and ordering the query.
    fn normalise(&self, url: Url) -> Result<Url, Box<dyn Error>> {
        match self.full_normalisation(url) {
            Ok(url) => Ok(url),
            Err(_) => Err(Box::new(NormaliseError("Normalisation went wrong.".into()))),
        }
    }
}

impl DefaultNormaliser {
    /// Takes a slice of functions which gets applied to the given tasks url.
    fn custom_normalisation<F>(&self, task: Task, f: F) -> Result<Task, ()> where F: FnOnce(Url) -> Result<Url, ()> {
        let mut new_url = task.url;

        new_url = f(new_url)?;

        Ok(Task {
            url: new_url
        })
    }

    /// Run through all implemented normalisation functions and applying those on the given url.
    fn full_normalisation(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        //Normalising by ordering the query in alphabetic order,
        //removes hash from url and changes encrypted to unencrypted.
        new_url = url_normalizer::normalize(new_url)?;
        /*
        new_url = self.removing_dots_in_path(new_url)?; //When creating an url it removes dots by default
        */
        new_url = self.scheme_and_host_to_lowercase(new_url)?;
        new_url = self.converting_encoded_triplets_to_upper(new_url)?;
        new_url = self.empty_path_to_slash(new_url)?;

        Ok(new_url)
    }

    ///Sets the scheme and host to lowercase
    fn scheme_and_host_to_lowercase(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        let scheme = new_url.scheme().to_lowercase();

        if let Some(host) = new_url.host_str() {
            let host = host.to_lowercase();

            match new_url.set_host(Option::Some(host.as_str())) {
                Ok(_) => {}
                Err(e) => {
                    println!("Couldn't set the host: {}", e);
                }
            }
        }

        new_url.set_scheme(scheme.as_str())?;

        Ok(new_url)
    }


    ///Removing the "../" and "./" notations from the path
    #[allow(dead_code)]
    fn removing_dots_in_path(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        let mut path = new_url.path().to_string();
        path = path.replace("../", "").replace("./", "");

        new_url.set_path(path.as_str());

        Ok(new_url)
    }

    /// Converting encoded triplets to uppercase, example:
    /// From: "http://example.com/foo%2a"
    /// To: "http://example.com/foo%2A"
    fn converting_encoded_triplets_to_upper(&self, url: Url) -> Result<Url, ()> {
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

        // Parse the builded string as an url for return
        match Url::parse(str_build.as_str()) {
            Ok(url) => Ok(url),
            Err(e) => {
                Err(()) //TODO handling Error
            }
        }
    }

    /// Converting an empty path to a slash. Example:
    /// From: "http://example.com"
    /// To: "http://example.com/"
    fn empty_path_to_slash(&self, url: Url) -> Result<Url, ()> {
        let new_url = url;
        let url_as_str = new_url.as_str();

        if url_as_str.ends_with("/") || !new_url.path().is_empty() {
            Ok(new_url)
        } else {
            url_as_str.to_string().push_str("/");

            match Url::parse(url_as_str) {
                Ok(url) => Ok(url),
                Err(e) => {
                    Err(()) //TODO Handling of Error
                }
            }
        }
    }
}


#[derive(Debug)]
struct NormaliseError(String);

impl fmt::Display for NormaliseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for NormaliseError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_normalisation0() {
        let normaliser = DefaultNormaliser;
        let expected_url = "shoot:shoot:shoot";
        let mut test_task = Task {
            url: Url::parse("HTTPS://user:pass@sub.HOST.cOm").unwrap()
        };


        let shoot = |url: Url| { Ok(Url::parse("shoot:shoot:shoot").unwrap()) };

        test_task = normaliser.custom_normalisation(test_task, shoot).unwrap();

        assert_eq!(test_task.url.to_string(), expected_url);
    }

    #[test]
    fn test_empty_path_to_slash() {
        let normaliser = DefaultNormaliser;

        let expected_url = "http://example.com/";
        let mut test_task = Task {
            url: Url::parse("http://example.com").unwrap()
        };

        let test_url = normaliser.empty_path_to_slash(test_task.url).unwrap();

        assert_eq!(test_url.to_string(), expected_url);
    }

    #[test]
    fn test_converting_encoded_triplets_to_upper() {
        let normaliser = DefaultNormaliser;

        let expected_url = "http://example.com/foo%2A";
        let mut test_task = Task {
            url: Url::parse("http://example.com/foo%2a").unwrap()
        };

        let test_url = normaliser.converting_encoded_triplets_to_upper(test_task.url).unwrap();

        assert_eq!(test_url.to_string(), expected_url);
    }

    #[test]
    fn test_scheme_and_host_to_lowercase0() {
        let normaliser = DefaultNormaliser;

        let expected_url = "https://user:pass@sub.host.com:8080/p/a/t/h?query=string#hash";

        let test_task = Task {
            url: Url::parse("HTTPS://user:pass@sub.HOST.cOm:8080/p/a/t/h?query=string#hash")
                .unwrap(),
        };

        let test_url = normaliser
            .scheme_and_host_to_lowercase(test_task.url)
            .unwrap();

        assert_eq!(test_url.to_string(), expected_url);
    }

    #[test]
    fn test_scheme_and_host_to_lowercase1() {
        let normaliser = DefaultNormaliser;
        let test_task = Task {
            url: Url::parse("HTTPS://user:pass@sub.HOST.cOm:8080/p/a/t/h?query=string#hash")
                .unwrap(),
        };

        let test_url = normaliser
            .scheme_and_host_to_lowercase(test_task.url)
            .unwrap();

        let test_scheme = test_url.scheme();
        let test_host = test_url.host_str().unwrap();

        let expected_scheme = "https";
        let expected_host = "sub.host.com";

        assert_eq!(test_scheme, expected_scheme);
    }

    #[test]
    fn test_scheme_and_host_to_lowercase2() {
        let normaliser = DefaultNormaliser;
        let test_task = Task {
            url: Url::parse("HTTPS://user:pass@sub.HOST.cOm:8080/p/a/t/h?query=string#hash")
                .unwrap(),
        };

        let test_url = normaliser
            .scheme_and_host_to_lowercase(test_task.url)
            .unwrap();

        let test_scheme = test_url.scheme();
        let test_host = test_url.host_str().unwrap();

        let expected_scheme = "https";
        let expected_host = "sub.host.com";

        assert_eq!(test_host, expected_host);
    }

    #[test]
    fn test_scheme_and_host_to_lowercase3() {
        let normaliser = DefaultNormaliser;
        let test_task = Task {
            url: Url::parse("urn:oasis:names:specification:docbook:dtd:xml:4.1.2")
                .unwrap(),
        };

        let test_url = normaliser
            .scheme_and_host_to_lowercase(test_task.url)
            .unwrap();

        assert_eq!(test_url.has_host(), false)
    }
}
