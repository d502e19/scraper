use std::fs;
use crate::traits::Filter;
use crate::task::Task;
use std::fs::File;
use std::io::{Read, BufReader, BufRead};

pub(crate) struct Whitelist {
   ok_urls: Vec<String>
}

impl Whitelist {
    pub fn new() -> Self {
        Whitelist { ok_urls: Whitelist::read_from_whitelist_file() }
    }

    fn read_from_whitelist_file() -> Vec<String> {
        let file = File::open("whitelist.txt").unwrap();
        let buf = BufReader::new(file);

        let mut data: Vec<String> = buf.lines()
            .map(|l| l.unwrap())
            .map(|l| {
                l.trim().to_string()
            })
            .collect();

        return data;
    }
}


impl Filter for Whitelist {
    fn filter(&self, task: &Task) -> bool {
        if self.ok_urls.contains(&task.url.to_string()) {
            return true
        } else {
            return false
        }
    }
}