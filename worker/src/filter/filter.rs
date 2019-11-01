use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use crate::task::Task;
use crate::traits::Filter;

/// Functions as a filter but is doing nothing
pub(crate) struct NoFilter;

impl Filter for NoFilter { fn filter(&self, task: &Task) -> bool {true} }


/// Contains a Vec of all the entries in the whitelist.txt
pub(crate) struct Whitelist {
    whitelist: Vec<String>,
    path: String,
}

//TODO arg for not using whitelist
//TODO add error(handling)
impl Whitelist {
    pub fn new(path: String) -> Self {
        Whitelist {
            whitelist: Whitelist::read_from_whitelist_file(path),
            path,
        }
    }

    /// Reads from whitelist.txt and returns the entries as a Vec.
    /// Called by the new() on whitelist struct
    fn read_from_whitelist_file(path: String) -> Vec<String> {
        let file = File::open(path).unwrap(); // handle unwrap better
        let buf = BufReader::new(file);

        let mut data: Vec<String> = buf.lines()
            .map(|l| l.unwrap())
            .map(|l| {
                l.trim().to_string()
            })
            .collect();

        return data;
    }

    /// Appends a url to whitelist if it is not already in the whitelist
    fn write_to_whitelist_file(url: String, path: String) -> bool {
        // Remove www. from url
        let shortened_url = url.replacen("www.", "", 1);

        // If url is not in the whitelist file, append it to the whitelist file
        if !Whitelist::read_from_whitelist_file(path).contains(&shortened_url) {

            // Open whitelist file
            let mut file = OpenOptions::new().append(true)
                .open("src/filter/whitelist.txt")
                .expect("cannot open whitelist file");

            // Write url to whitelist file, with newline
            file.write(format!("\n{}", shortened_url).as_bytes()).
                expect("write to whitelist file failed");

            return true;
        }
        // Return false if url already is in the whitelist file
        return false;
    }
}

impl Filter for Whitelist {
    /// Takes a task and returns true if the task's url exists in whitelist file, else false
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
            // If url is not in whitelist, append to whitelist. Assumes all links are good.
            Whitelist::write_to_whitelist_file(host_url, (&self.path).to_string());
            return false;
        }
        // If no host url in task, e.g if task is an email address, return false
        eprintln!("[filter] Not possible to find host url in task url: {}", task.url);
        return false;
    }
}