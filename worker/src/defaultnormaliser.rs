use crate::task::Task;
use crate::traits::Normaliser;

use std::error::Error;
use std::fmt;
use url::Url;
use url_normalizer;

pub struct DefaultNormaliser;

impl DefaultNormaliser {
    fn full_normalisation(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        //Normalising by ordering the query in alphabetic order,
        //removes hash from url and changes encrypted to unencrypted.
        new_url = url_normalizer::normalize(new_url)?;
        new_url = self.scheme_and_host_to_lowercase(new_url)?;

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
    // Since the parsing when creating a Url is removing the "./" and
    // "../" notation this method is obsolete and not included in full_normalisation()
    fn removing_dots_in_path(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        let mut path = new_url.path().to_string();
        path = path.replace("../", "").replace("./", "");

        new_url.set_path(path.as_str());

        Ok(new_url)
    }
}

impl Normaliser for DefaultNormaliser {
    /// Normalising the tasks URL by setting scheme and path to lowercase,
    /// removing the dot in path, removes hash from url and ordering the query.
    fn normalise(&self, task: Task) -> Result<Task, Box<dyn Error>> {
        match self.full_normalisation(task.url) {
            Ok(url) => Ok(Task { url }),
            Err(_) => Err(Box::new(NormaliseError("Normalisation went wrong.".into()))),
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
