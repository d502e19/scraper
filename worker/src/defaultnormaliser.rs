use crate::traits::Normaliser;
use crate::task::Task;

use url::{Url, ParseError};
use url_normalizer::normalize;
use std::error::Error;
use tokio::io::ErrorKind;
use std::fmt;
use test::Options;

pub struct DefaultNormaliser {}

impl DefaultNormaliser {
    fn full_normalisation(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        //Normalising by ordering the query in alphabetic order,
        //removes hash from url and changes encrypted to unencrypted.
        new_url = url_normalizer::normalize(new_url).unwrap();

        removing_dots_in_path(new_url)
    }

    fn scheme_and_host_to_lowercase(&self, url:Url) -> Result<Url, ()>{
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

        url_new.set_path(path)
    }
}

impl Normaliser for DefaultNormaliser {
    fn normalise(&self, task: Task) -> Result<Task, Box<dyn Error>> {
        let url = task.url;

        match full_normalisation(url) {
            Ok(new_url) => {
                Ok(
                    Task { url: Url::parse(new_url) }
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

impl Error for NormaliseError {}
use std::fmt::Error;
use url::Url;
use url_normalizer::normalize;


pub struct DefaultNormaliser {}

impl Normaliser {}

