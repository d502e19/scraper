use crate::errors::ArchiveResult;
use crate::traits::Archive;

/// A Void is an Archive that does not store the given data
pub struct Void;

impl<D> Archive<D> for Void {
    fn archive_content(&self, _content: D) -> ArchiveResult<()> {
        Ok(())
    }
}
