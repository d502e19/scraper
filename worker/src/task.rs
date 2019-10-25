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
    use url::Url;

    use crate::task;
    use crate::task::Task;

    /// Test if serialisation and deserialisation does not change the Task
    #[test]
    fn serialise_deserialise_success() {
        let task1 = task::Task { url: Url::parse("http://aau.dk/").unwrap() };
        let task1_serialised = task1.serialise();
        let task1_regen = Task::deserialise(task1_serialised);
        assert_eq!(task1, task1_regen);
    }

    /// Test if serialisation and deserialisation on two different tasks still makes them unequal
    #[test]
    fn serialise_deserialise_failure() {
        let task1 = task::Task { url: Url::parse("http://aau.dk/").unwrap() };
        let task2 = task::Task { url: Url::parse("http://aau2.dk/").unwrap() };

        let task1_serialised = task1.serialise();
        let task2_serialised = task2.serialise();
        let task1_regen = Task::deserialise(task1_serialised);
        let task2_regen = Task::deserialise(task2_serialised);
        assert_ne!(task1_regen, task2_regen);
    }

    /// Equality between two identical URLs
    #[test]
    fn normalisation_equality_1() {
        let task1 = task::Task { url: Url::parse("http://aau.dk").unwrap() };
        let task2 = task::Task { url: Url::parse("http://aau.dk").unwrap() };
        assert_eq!(task1, task2)
    }

    /// Equality between all-caps and all lower-caps
    #[test]
    fn normalisation_equality_2() {
        let task1 = task::Task { url: Url::parse("http://aau.dk").unwrap() };
        let task2 = task::Task { url: Url::parse("HTTP://AAU.DK/").unwrap() };
        assert_eq!(task1, task2)
    }

    /// Equality between implicit port of protocol and explicit
    #[test]
    fn normalisation_equality_3() {
        let task1 = task::Task { url: Url::parse("http://aau.dk").unwrap() };
        let task2 = task::Task { url: Url::parse("http://aau.dk:80").unwrap() };
        assert_eq!(task1, task2)
    }

    /// Simple inequality between two different domains
    #[test]
    fn normalisation_inequality_1() {
        let task1 = task::Task { url: Url::parse("http://aau.dk").unwrap() };
        let task2 = task::Task { url: Url::parse("http://aaau.dk/").unwrap() };
        assert_ne!(task1, task2)
    }

    /// Domain labels change semantics and as such these two URLs are inequal
    #[test]
    fn normalisation_inequality_2() {
        let task1 = task::Task { url: Url::parse("https://aau.dk").unwrap() };
        let task2 = task::Task { url: Url::parse("https://www.aau.dk").unwrap() };
        assert_ne!(task1, task2)
    }

    /// Inequality of different port to implicit port of protocol
    #[test]
    fn normalisation_inequality_3() {
        let task1 = task::Task { url: Url::parse("http://aau.dk").unwrap() };
        let task2 = task::Task { url: Url::parse("https://aau.dk:81").unwrap() };
        assert_ne!(task1, task2)
    }
}