
pub struct Task {
    pub url: String,
}

impl Task {
    pub fn serialise(self) -> Vec<u8> {
        self.url.into_bytes()
    }

    pub fn deserialise(data: Vec<u8>) -> Self {
        Task { url: String::from_utf8(data).unwrap() }
    }
}