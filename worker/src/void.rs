use crate::errors::ArchiveResult;
use crate::traits::Archive;

///Functions as an Archive but is doing nothing with the content.
pub struct Void;

impl<D> Archive<D> for Void {
    fn archive_content(&self, _content: D) -> ArchiveResult<()> {
        Ok(())
    }
}
