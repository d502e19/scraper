use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};

use crate::task::Task;
use crate::traits::Filter;

/// Functions as a filter but is doing nothing, thereby allowing all urls to be visited
pub(crate) struct NoFilter;

impl Filter for NoFilter {
    fn filter(&self, tasks: Vec<Task>) -> Vec<Task> { tasks }
}

/// A Blacklist is a Filter that will cull specific Urls
pub(crate) struct Blacklist {
    urls: Vec<String>,
}

impl Blacklist {
    /// Construct a Blacklist from a file or blacklisted Url substrings
    pub fn new(path: String) -> Self {
        Blacklist {
            urls: read_filter_from_file((&path).to_string()),
        }
    }

    /// Construct a Blacklist from a Vec of Url substrings
    #[cfg(test)]
    fn new_from_vec(urls: Vec<String>) -> Self {
        Blacklist { urls }
    }
}

impl Filter for Blacklist {
    /// Removes all tasks which url is blacklisted
    fn filter(&self, mut tasks: Vec<Task>) -> Vec<Task> {
        return tasks.drain(..).filter(|task| {
            // If no host url in task, e.g if task is an email address, return false
            if let Some(host_url) = task.url.host_str() {
                let host_url = host_url.to_string();
                // Check if the host_url contains a blacklisted substring
                for url in &self.urls {
                    if !host_url.contains(url) {
                        return true;
                    }
                }
            }
            return false;
        }).collect();
    }
}

/// A Whitelist is a Filter that only allows certain Urls
pub(crate) struct Whitelist {
    urls: Vec<String>,
}

impl Whitelist {
    /// Construct a Whitelist from a file or whitelisted Url substrings
    pub fn new(path: String) -> Self {
        Whitelist {
            urls: read_filter_from_file((&path).to_string()),
        }
    }
    /// Construct a Whitelist from a Vec of Url substrings
    #[cfg(test)]
    fn new_from_vec(urls: Vec<String>) -> Self {
        Whitelist { urls }
    }
}

impl Filter for Whitelist {
    /// Removes all tasks which url is not whitelisted
    fn filter(&self, mut tasks: Vec<Task>) -> Vec<Task> {
        return tasks.drain(..).filter(|task| {
            // If no host url in task, e.g if task is an email address, return false
            if let Some(host_url) = task.url.host_str() {
                let host_url = host_url.to_string();
                // Check if the host_url contains a whitelisted substring
                for url in &self.urls {
                    if host_url.contains(url) {
                        return true;
                    }
                }
            }
            return false;
        }).collect();
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
            .open(&path)
            .expect(format!("Could not open file; {:?} for writing filter list", &path).as_str());

        // Write url to file, with newline
        file.write(format!("\n{}", shortened_url).as_bytes())
            .expect(format!("Could not write to file; {:?} for writing filter list", &path).as_str());

        return true;
    }
    // Return false if url already is in the file
    return false;
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::filter::filter::{Blacklist, NoFilter, Whitelist};
    use crate::task;
    use crate::task::Task;
    use crate::traits::Filter;

    /// Setup vector of Urls as Strings for testing
    fn get_predefined_list() -> Vec<String> {
        // Create a vec of Strings, by mapping to strings and lastly collecting
        vec!["reddit.com", "bbc.co.uk", "dr.dk"].iter().map(|f| f.to_string()).collect()
    }

    /// Test that looking up a Url that is contained in whitelist returns true
    #[test]
    fn whitelist_test_01() {
        let whitelist = Whitelist::new_from_vec(get_predefined_list());
        let task = task::Task { url: Url::parse("http://reddit.com").unwrap() };
        let tasks: Vec<Task> = vec![task];

        let expected = task::Task { url: Url::parse("http://reddit.com").unwrap() };
        assert!(whitelist.filter(tasks).contains(&expected))
    }

    /// Test that looking up a Url that is NOT contained in whitelist returns false
    #[test]
    fn whitelist_test_02() {
        let whitelist = Whitelist::new_from_vec(get_predefined_list());
        let task = task::Task { url: Url::parse("http://tv2.dk").unwrap() };
        let tasks: Vec<Task> = vec![task];

        let expected = task::Task { url: Url::parse("http://tv2.dk").unwrap() };
        assert!(!whitelist.filter(tasks).contains(&expected))
    }

    /// Test that looking up a Url that is NOT contained in blacklist returns true
    #[test]
    fn whitelist_test_03() {
        let whitelist = Whitelist::new_from_vec(get_predefined_list());
        let task = task::Task { url: Url::parse("http://bbc.co.uk").unwrap() };
        let task1 = task::Task { url: Url::parse("http://okboomer.dk").unwrap() };
        let tasks: Vec<Task> = vec![task, task1];

        let expected = task::Task { url: Url::parse("http://bbc.co.uk").unwrap() };
        assert!(whitelist.filter(tasks).contains(&expected))
    }

    /// Test that looking up a Url that is contained in blacklist returns false
    #[test]
    fn blacklist_test_01() {
        let blacklist = Blacklist::new_from_vec(get_predefined_list());
        let task = task::Task { url: Url::parse("http://reddit.com").unwrap() };
        let tasks: Vec<Task> = vec![task];

        let expected = task::Task { url: Url::parse("http://reddit.com").unwrap() };
        assert!(blacklist.filter(tasks).contains(&expected))
    }

    /// Test that looking up a Url that is NOT contained in blacklist returns true
    #[test]
    fn blacklist_test_02() {
        let blacklist = Blacklist::new_from_vec(get_predefined_list());
        let task = task::Task { url: Url::parse("http://tv2.dk").unwrap() };
        let tasks: Vec<Task> = vec![task];

        let expected = task::Task { url: Url::parse("http://tv2.dk").unwrap() };
        assert!(blacklist.filter(tasks).contains(&expected))
    }

    /// Test that looking up a Url that is NOT contained in blacklist returns true
    #[test]
    fn blacklist_test_03() {
        let blacklist = Blacklist::new_from_vec(get_predefined_list());
        let task = task::Task { url: Url::parse("http://reddit.com").unwrap() };
        let task1 = task::Task { url: Url::parse("http://okboomer.dk").unwrap() };
        let tasks: Vec<Task> = vec![task, task1];

        let expected = task::Task { url: Url::parse("http://okboomer.dk").unwrap() };
        assert!(blacklist.filter(tasks).contains(&expected))
    }

    /// Test that looking up a Url that is NOT contained in blacklist returns true
    #[test]
    fn nofilter_test_01() {
        let filter = NoFilter;

        let task = task::Task { url: Url::parse("http://tv2.dk").unwrap() };
        let tasks: Vec<Task> = vec![task];

        let expected = task::Task { url: Url::parse("http://tv2.dk").unwrap() };
        assert!(filter.filter(tasks).contains(&expected))
    }

    /// Test that looking up a Url that is NOT contained in blacklist returns true
    #[test]
    fn nofilter_test_02() {
        let filter = NoFilter;

        let task1 = task::Task { url: Url::parse("http://tv2.dk").unwrap() };
        let task2 = task::Task { url: Url::parse("http://okboomer.dk").unwrap() };
        let task3 = task::Task { url: Url::parse("http://tv2.dk").unwrap() };
        let tasks: Vec<Task> = vec![task1, task2, task3];

        let mut tasks_clone: Vec<Task> = Vec::new();
        for i in &tasks {
            tasks_clone.push(Task { url: Url::parse(i.url.as_str()).unwrap() })
        }

        assert_eq!(filter.filter(tasks), tasks_clone)
    }
}