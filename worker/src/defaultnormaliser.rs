use crate::traits::Normaliser;
use crate::task::Task;

use url::Url;
use url_normalizer;
use std::error::Error;
use std::fmt;

pub struct DefaultNormaliser;

impl DefaultNormaliser {
    fn full_normalisation(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        //Normalising by ordering the query in alphabetic order,
        //removes hash from url and changes encrypted to unencrypted.
        new_url = url_normalizer::normalize(new_url)?;

        new_url = self.scheme_and_host_to_lowercase(new_url)?;
        new_url = self.removing_dots_in_path(new_url)?;

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
    fn removing_dots_in_path(&self, url: Url) -> Result<Url, ()> {
        let mut new_url = url;

        let mut path = new_url.path().to_string();
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

impl Error for NormaliseError {}


