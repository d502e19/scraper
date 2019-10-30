use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};

use crate::task::Task;
use crate::traits::Filter;

/// Contains a Vec of all the entries in the whitelist.txt
pub(crate) struct Whitelist {
    whitelist: Vec<String>
}

impl Whitelist {
    pub fn new() -> Self {
        Whitelist { whitelist: Whitelist::read_from_whitelist_file() }
    }

    /// Reads from whitelist.txt and returns the entries as a Vec.
    /// Called by the new() on whitelist struct
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
    /// Takes a task and returns true or false, whether or not the url in the task is found in the
    /// whitelist.txt
    fn filter(&self, task: &Task) -> bool {
        // If there is a host string assign this to host-url, else return false
        if let Some(host_url) = task.url.host_str() {
            let host_url = host_url.to_string();
            /* Iterates through whitelist and sees if the host_url contains a substring of any
            entry in the whitelist, therefore all paths and sub-domains*/
            for url in &self.whitelist {
                if host_url.contains(url) {
                    return true;
                }
            }
            //todo add to whitelist
            println!(">>>>>>>>>>FOUND NEW HOST_URL: {}<<<<<<<<<<", host_url);
            return false;
        }
        eprintln!("Not possible to find host url in task url");
        return false;
    }
}