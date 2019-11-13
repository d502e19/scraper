use redis::{FromRedisValue, RedisError, RedisWrite, ToRedisArgs, Value};
use std::io::{Error, ErrorKind};
use url::Url;

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
    pub fn deserialise(data: Vec<u8>) -> Result<Self, Error> {
        // checks if there is an error when changing data to a string
        let data_to_string_res = String::from_utf8(data);
        match data_to_string_res {
            Ok(data) => {
                let url_res = Url::parse(&data);
                // checks if there is an error when parsing the url
                match url_res {
                    Ok(url) => Ok(Task { url }),
                    Err(_) => Err(Error::new(ErrorKind::InvalidInput, "failed to deserialise"))
                }
            }
            Err(_) => Err(Error::new(ErrorKind::InvalidData, "failed to deserialise"))
        }
    }
}

impl PartialEq for Task {
    // Two tasks are equal if they have the same Url
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

// Allows Redis to automatically serialise Task into raw bytes with type inference
impl ToRedisArgs for &Task {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite,
    {
        out.write_arg(self.url.as_str().as_bytes())
    }
}

// Allows Redis to automatically deserialise Task from raw bytes with type inference
impl FromRedisValue for Task {
    fn from_redis_value(v: &Value) -> Result<Self, RedisError> {
        match *v {
            Value::Data(ref bytes) => {
                let result = Task::deserialise(bytes.to_owned());
                match result {
                    Ok(task) => Ok(task),
                    Err(_) => Err(RedisError::from(Error::new(ErrorKind::Other, "failed to deserialise")))
                }
            }
            _ => Err(RedisError::from(Error::new(
                ErrorKind::Other,
                "Response could not be translated to a task",
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::task;
    use crate::task::Task;

    /// Test if serialisation and deserialisation does not change the Task
    #[test]
    fn deserialise_success01() {
        let task = task::Task { url: Url::parse("http://aub.dk/").unwrap() };
        let task_serialise = task.serialise();
        let task_deserialise = Task::deserialise(task_serialise).unwrap();
        assert_eq!(task, task_deserialise);
    }

    // Test if fail to deserialise when task contains non utf8 character
    #[test]
    fn deserialise_fail01() {
        let task = "https://www.�.com/".as_bytes();
        let task_deserialise = Task::deserialise(task.to_vec());
        assert!(task_deserialise.is_err())
    }
}