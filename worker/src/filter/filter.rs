use std::fs;
use crate::traits::Filter;
use crate::task::Task;
use std::fs::File;
use std::io::Read;

pub(crate) struct Whitelist {
   ok_urls: Vec<u8>
}

impl Whitelist {
    pub fn new() -> Self {
        Whitelist { ok_urls: Whitelist::read_from_whitelist_file() }
    }

    fn read_from_whitelist_file() -> Vec<u8> {
        let mut file = File::open("whitelist.txt").expect("Could not read from whitelist file");

        let mut data = Vec::new();
        file.read_to_end(&mut data);

        return data;
    }
}


impl Filter<Vec<u8>> for Whitelist {
    fn filter(&self, task: &Task) -> bool {
        if self.ok_urls.contains(&task.url.as_str().as_ref()) {
            return true
        } else {
            return false
        }
    }
}