use anyhow::Result;

mod deflate;
mod text;

pub use self::deflate::DeflatableFiles;
pub use self::text::TextFiles;

#[derive(PartialEq)]
pub enum Consuming {
    /// The criteria is still working, and may consume more data
    Working,
    /// The criteria has made a determination and does not need more data
    Done,
}

#[derive(PartialEq)]
pub enum Selection {
    /// The file fits the selection criteria for the criteria
    Select,
    /// The file does not fit the selection criteria for the criteria
    Ignore,
}

pub trait Criteria {
    /// Initialize the criteria, resetting any previous state
    fn initialize(&mut self);

    /// Process block of data and return value signifying if more data is useful
    fn process(&mut self, data: &[u8]) -> Result<Consuming>;

    /// Returns the final determination
    fn finalize(&mut self) -> Result<Selection>;
}
