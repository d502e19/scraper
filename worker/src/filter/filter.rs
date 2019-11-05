use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use crate::task::Task;
use crate::traits::Filter;

/// Functions as a filter but is doing nothing, thereby allowing all urls to be visited
pub(crate) struct NoFilter;

impl Filter for NoFilter {
    fn filter(&self, task: &Task) -> bool {
        true
    }
}

/// Contains a Vec of all the entries in the path given
pub(crate) struct Blacklist {
    urls: Vec<String>,
}

impl Blacklist {
    /// Constructor for blacklist struct. Automatically reads from path and puts urls into struct
    pub fn new(path: String) -> Self {
        Blacklist {
            urls: read_filter_from_file((&path).to_string()),
        }
    }
    /// Private constructor for unit testing
    #[cfg(test)]
    fn new_from_vec(urls: Vec<String>) -> Self {
        Blacklist { urls }
    }
}

impl Filter for Blacklist {
    /// Takes a task and returns false if the task's url exists in blacklist file, else true
    fn filter(&self, task: &Task) -> bool {
        // Assign host url as string to variable if possible, else return true
        if let Some(host_url) = task.url.host_str() {
            let host_url = host_url.to_string();
            /* Iterates through blacklist and sees if the host_url contains a substring of any
            entry in the list, therefore checks all paths and sub-domains*/
            for url in &self.urls {
                if host_url.contains(url) {
                    return false;
                }
            }
            return true;
        }
        return true;
    }
}

/// Contains a Vec of all the entries in the path given
pub(crate) struct Whitelist {
    urls: Vec<String>,
}

impl Whitelist {
    /// Constructor for whitelist struct. Automatically reads from path and puts urls into struct
    pub fn new(path: String) -> Self {
        Whitelist {
            urls: read_filter_from_file((&path).to_string()),
        }
    }
    /// Private constructor for unit testing
    #[cfg(test)]
    fn new_from_vec(urls: Vec<String>) -> Self {
        Whitelist { urls }
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

/// Reads from file and returns the entries as a Vec.
/// Called by the new() on filter structs
fn read_filter_from_file(path: String) -> Vec<String> {
    let file = File::open(&path).expect(format!("Could not open file: {:?}", &path).as_str());
    let buf = BufReader::new(file);

    // Read each line from file, trim and collect them into a vector of strings
    let data: Vec<String> = buf
        .lines()
        .map(|l| l.unwrap())
        .map(|l| l.trim().to_string())
        .collect();

    return data;
}

/// Appends a url to a file, checks if it already exists before doing so
/// Currently not used, but could have some use
#[allow(dead_code)] // Suppressing compiler warning on unused function
fn write_to_filter_file(url: String, path: String) -> bool {
    // Remove www. from url
    let shortened_url = url.replacen("www.", "", 1);

    // If url is not in the file, append it
    if read_filter_from_file((&path).to_string()).contains(&shortened_url) {
        // Open file
        let mut file = OpenOptions::new()
            .append(true)
            .open(path)
            .expect("cannot open file");

        // Write url to file, with newline
        file.write(format!("\n{}", shortened_url).as_bytes())
            .expect("write to file failed");

        return true;
    }
    // Return false if url already is in the file
    return false;
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::filter::filter::{Blacklist, Whitelist};
    use crate::task::Task;
    use crate::traits::Filter;

    /// Setup vector of URLs as Strings for testing
    fn get_predefined_list() -> Vec<String> {
        // Create a vec of Strings, by mapping to strings and lastly collecting
        vec!["reddit.com", "bbc.co.uk", "dr.dk"].iter().map(|f| f.to_string()).collect()
    }

    /// Test that looking up a URL that is contained in whitelist returns true
    #[test]
    fn whitelist_test_01() {
        let whitelist = Whitelist::new_from_vec(get_predefined_list());
        assert!(whitelist.filter(&Task { url: Url::parse("http://reddit.com").unwrap() }))
    }

    /// Test that looking up a URL that is NOT contained in whitelist returns false
    #[test]
    fn whitelist_test_02() {
        let whitelist = Whitelist::new_from_vec(get_predefined_list());
        assert!(!whitelist.filter(&Task { url: Url::parse("http://tv2.dk").unwrap() }))
    }

    /// Test that looking up a URL that is contained in blacklist returns false
    #[test]
    fn blacklist_test_01() {
        let blacklist = Blacklist::new_from_vec(get_predefined_list());
        assert!(!blacklist.filter(&Task { url: Url::parse("http://reddit.com").unwrap() }))
    }

    /// Test that looking up a URL that is NOT contained in blacklist returns true
    #[test]
    fn blacklist_test_02() {
        let blacklist = Blacklist::new_from_vec(get_predefined_list());
        assert!(blacklist.filter(&Task { url: Url::parse("http://tv2.dk").unwrap() }))
    }
}