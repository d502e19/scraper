use std::str::from_utf8;

use redis::{FromRedisValue, RedisError, RedisWrite, ToRedisArgs, Value};

#[derive(Hash, Eq, Debug)]
pub struct Task {
    pub url: String,
}

impl Task {
    pub fn serialise(&self) -> Vec<u8> {
        self.url.as_bytes().to_vec()
    }

    pub fn deserialise(data: Vec<u8>) -> Self {
        Task {
            url: String::from_utf8(data).unwrap(),
        }
    }
}

impl PartialEq for Task {
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
        out.write_arg(self.url.as_bytes())
    }
}

// Allows Redis to automatically deserialise Task from raw bytes with type inference
impl FromRedisValue for Task {
    fn from_redis_value(v: &Value) -> Result<Self, RedisError> {
        match *v {
            Value::Data(ref bytes) => Ok(Task {
                url: from_utf8(bytes)?.to_string(),
            }),
            _ => panic!((
                "Response type could not be translated to a Task.",
                format!("Response was {:?}", v)
            )),
        }
    }
}
