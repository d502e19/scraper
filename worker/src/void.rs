use crate::traits::Archive;
use crate::errors::ArchiveResult;

///Functions as an Archive but is doing nothing with the content.
pub struct Void {}

impl<D> Archive<D> for Void {
    fn archive_content(&self, content: D) -> ArchiveResult<()> {
        Ok(())
    }
}