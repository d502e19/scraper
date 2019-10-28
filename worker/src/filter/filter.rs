



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

    }
}