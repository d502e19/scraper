use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use crate::task::Task;
use crate::traits::Filter;

//TODO add impl for other filters, and use these in main if parser arguments is set
// FIXME restructure code, no writes
/* FIXME These structs will be used at a later point when worker can accept multiple different filters
/// Functions as a filter but is doing nothing
pub(crate) struct NoFilter;
impl Filter for NoFilter { fn filter(&self, task: &Task) -> bool {true} }

/// Contains a Vec of all the entries in the blacklist.txt
pub(crate) struct Blacklist {
    urls: Vec<String>,
    path: String,
}*/

/// Reads from file and returns the entries as a Vec.
/// Called by the new() on filter struct
fn read_from_filter_file(path: String) -> Vec<String> {
    let file = File::open(path).unwrap(); // handle unwrap better
    let buf = BufReader::new(file);

    let data: Vec<String> = buf
        .lines()
        .map(|l| l.unwrap())
        .map(|l| l.trim().to_string())
        .collect();

    return data;
}

/// Appends a url to whitelist if it is not already in the whitelist
/// Currently not used, but could have some use
fn write_to_filter_file(url: String, path: String) -> bool {
    // Remove www. from url
    let shortened_url = url.replacen("www.", "", 1);

    // If url is not in the whitelist file, append it to the whitelist file
    if read_from_filter_file((&path).to_string()).contains(&shortened_url) {
        // Open whitelist file
        let mut file = OpenOptions::new()
            .append(true)
            .open(path)
            .expect("cannot open file");

        // Write url to whitelist file, with newline
        file.write(format!("\n{}", shortened_url).as_bytes())
            .expect("write to file failed");

        return true;
    }
    // Return false if url already is in the file
    return false;
}

/// Contains a Vec of all the entries in the whitelist.txt and path to this file
pub(crate) struct Whitelist {
    urls: Vec<String>,
    path: String,
    activated: bool,
}

impl Whitelist {
    pub fn new(path: String, activated: bool) -> Self {
        Whitelist {
            urls: read_from_filter_file((&path).to_string()),
            path,
            activated,
        }
    }
}

impl Filter for Whitelist {
    /// Takes a task and returns true if the task's url exists in whitelist file, else false
    fn filter(&self, task: &Task) -> bool {
        /* FIXME: Hotfix to allow using no filter at all. If "filter-enable" is set to false in parser argument,
        this function just returns true. This field in struct will be removed at later point,
        if-statement placed here before rest of logic in function to ease the removal of hotfix later on*/
        if !self.activated {
            return true;
        }

        // If there is a host string assign this to host-url, else return false
        if let Some(host_url) = task.url.host_str() {
            let host_url = host_url.to_string();
            /* Iterates through whitelist and sees if the host_url contains a substring of any
            entry in the whitelist, therefore all paths and sub-domains*/
            for url in &self.urls {
                if host_url.contains(url) {
                    return true;
                }
            }
            return false;
        }
        // If no host url in task, e.g if task is an email address, return false
        return false;
    }
}
