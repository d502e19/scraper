
use url::{Url, ParseError};
use url_normalizer::normalize;
use std::error::Error;
use tokio::io::ErrorKind;
use std::fmt;
use futures::future::ok;

pub struct DefaultNormaliser {}

impl DefaultNormaliser {
    fn full_normalisation(&self, url: Url) -> Result<Url, ()> {
        let new_url = url;

        //Normalising by ordering the query in alphabetic order,
        //removes hash from url and changes encrypted to unencrypted.
        url_normalizer::normalize(new_url)
    }

    fn scheme_and_host_to_lowercase(&self, url:Url) -> Result<Url, ()>{
        let mut new_url = url;

        let host = new_url.host_str().unwrap().to_lowercase();
        let scheme = new_url.scheme().to_lowercase();

        new_url.set_scheme(&scheme);
        new_url.set_host(Option::Some(&host));

        Ok(new_url)

    }

    fn removing_dots_in_path(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        let mut path = new_url.path().to_string();
        path = path
            .replace("../", "")
            .replace("./", "");

        new_url.set_path(&path);

        Ok(new_url)
    }
}


#[derive(Debug)]
struct NormaliseError(String);

impl fmt::Display for NormaliseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

