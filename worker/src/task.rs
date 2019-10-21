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

#[cfg(test)]
mod tests {
    use crate::task;
    use url::Url;
    use crate::task::Task;

    #[test]
    /// Test if serialisation and deserialisation does not change the Task
    fn serialise_deserialise() {
        let task1 = task::Task {
            url: Url::parse("http://aau.dk/").unwrap(),
        };
        let task1_serialised = task1.serialise();
        let task1_regen = Task::deserialise(task1_serialised);
        assert_eq!(task1, task1_regen);
    }

    #[test]
    fn normalisation_equality() {
        let task1 = task::Task { url: Url::parse("http://aau.dk").unwrap() };
        let task2 = task::Task { url: Url::parse("http://aau.dk/").unwrap() };
        assert_eq!(task1, task2)
    }
}