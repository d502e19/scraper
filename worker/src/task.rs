use url::Url;
use crate::errors::{ManagerError, ManagerErrorKind, ManagerResult};

/// Tasks are the workload instances assigned to Workers. It describes a single Url that needs
/// to be resolved by the web scraper.
#[derive(Hash, Eq, Debug)]
pub struct Task {
    pub url: Url,
}

impl Task {
    /// Serialise the Task into bytes which makes it easier to transfer
    pub fn serialise(&self) -> Vec<u8> {
        self.url.as_str().as_bytes().to_vec()
    }

    /// Deserialise a series of bytes into a Task
    pub fn deserialise(data: Vec<u8>) -> ManagerResult<Self> {
        let data_to_string_res = String::from_utf8(data);
        // checks if there is an error when changing data to a string
        match data_to_string_res {
            Ok(data) => {
                let url_res = Url::parse(&data) ;
                // checks if there is an error when parsing the url
                match url_res {
                    Ok(url) => Ok(Task { url }),
                    Err(e) => Err(ManagerError::new(ManagerErrorKind::InvalidTask, "failed to deserialise", Some(Box::new(e))))
                }
            },
            Err(e) => Err(ManagerError::new(ManagerErrorKind::InvalidTask, "failed to deserialise", Some(Box::new(e))))
        }
    }
}

impl PartialEq for Task {
    // Two tasks are equal if they have the same Url
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
        let task1_regen = Task::deserialise(task1_serialised).unwrap();
        assert_eq!(task1, task1_regen);
    }

    /// Test if serialisation and deserialisation on two different tasks still makes them unequal
    #[test]
    fn serialise_deserialise_failure() {
        let task1 = task::Task { url: Url::parse("http://aau.dk/").unwrap() };
        let task2 = task::Task { url: Url::parse("http://aau2.dk/").unwrap() };

        let task1_serialised = task1.serialise();
        let task2_serialised = task2.serialise();
        let task1_regen = Task::deserialise(task1_serialised).unwrap();
        let task2_regen = Task::deserialise(task2_serialised).unwrap();
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

    // deserialise casts an error when data is not an url
    #[test]
    fn deserialise_fail_01() {
        let task = "mail@aau.dk".as_bytes();
        let task_deserialise = Task::deserialise(task.to_vec());
        assert!(task_deserialise.is_err())
    }

    // deserialise casts an error if the url contains anything that is not utf-8
    #[test]
    fn deserialise_fail_02() {
        let task = "https://www.�.com".as_bytes();
        let task_deserialise = Task::deserialise(task.to_vec());
        assert!(task_deserialise.is_err())
    }
}