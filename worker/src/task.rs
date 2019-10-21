use url::Url;

#[derive(Hash, Eq, Debug)]
pub struct Task {
    pub url: Url,
}

impl Task {
    pub fn serialise(&self) -> Vec<u8> {
        self.url.as_str().as_bytes().to_vec()
    }

    pub fn deserialise(data: Vec<u8>) -> Self {
        Task {
            // TODO; should implement error-checking on unwrapping both string from data and URL-parsing.
            url: Url::parse(String::from_utf8(data).unwrap().as_str()).unwrap(),
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}
