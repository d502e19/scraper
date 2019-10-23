use crate::traits::Archive;
use std::error::Error;

///Functions as an Archive but is doing nothing with the content.
pub struct Void;

impl<D> Archive<D> for Void {
    fn archive_content(&self, content: D) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
