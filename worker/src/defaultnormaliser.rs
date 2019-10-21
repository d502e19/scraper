use crate::traits::Normaliser;
use crate::task::Task;

use url::{Url, ParseError};
use url_normalizer::normalize;
use std::error::Error;
use tokio::io::ErrorKind;
use std::fmt;

pub struct DefaultNormaliser;

impl DefaultNormaliser {
    fn full_normalisation(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        //Normalising by ordering the query in alphabetic order,
        //removes hash from url and changes encrypted to unencrypted.
        url_normalizer::normalize(new_url)
    }

    fn scheme_and_host_to_lowercase(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        let host = url.host_str().unwrap().to_lowercase();
        let scheme = url.scheme().to_lowercase();

        new_url.set_scheme(&scheme);
        new_url.set_host(Option::Some(&host));

        Ok(new_url)
    }

    fn removing_dots_in_path(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        let mut path = url.path().to_string();
        path = path
            .replace("../", "")
            .replace("./", "");

        new_url.set_path(path.as_str());

        Ok(new_url)
    }
}

impl Normaliser for DefaultNormaliser {
    fn normalise(&self, task: Task) -> Result<Task, Box<dyn Error>> {

        match self.full_normalisation(task.url) {
            Ok(url) => {
                Ok(
                    Task {
                        url
                    }
                )
            }
            Err(_) => {
                Err(Box::new(NormaliseError("Normalisation went wrong.".into())))
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


