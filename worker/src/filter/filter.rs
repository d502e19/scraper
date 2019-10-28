



pub(crate) struct Whitelist {
   ok_urls: Vec<u8>
}

impl WhitelistFromFile {
    pub fn new() -> Self {
        WhitelistFromFile { ok_urls: ReadFromWHitelistFile() }
    }
}

impl WhitelistFromFile{
    fn ReadFromWhitelistFile() -> Vec<u8> {
        let contents = fs::read_to_string(whitelist).expect("Could not read from whitelist file");
        println!("contents from whitelist\n{}", contents)
    }
}

impl Filter<Vec<u8>> for Whitelist {
    fn filter(&self, task: &Task) -> bool{
        //TODO implement trait
    }
}