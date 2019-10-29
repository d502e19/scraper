use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use crate::task::Task;
use crate::traits::Filter;

pub(crate) struct Whitelist {
    ok_urls: Vec<String>
}

impl Whitelist {
    pub fn new() -> Self {
        Whitelist { ok_urls: Whitelist::read_from_whitelist_file() }
    }

    fn read_from_whitelist_file() -> Vec<String> {
        let file = File::open("src/filter/whitelist.txt").unwrap();
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