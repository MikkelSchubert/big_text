use anyhow::Result;

use crate::criteria::{Consuming, Criteria, Selection};

pub struct TextFiles {
    is_text: bool,
}

impl TextFiles {
    pub fn new() -> TextFiles {
        TextFiles { is_text: true }
    }
}

impl Criteria for TextFiles {
    fn initialize(&mut self) {
        self.is_text = true;
    }

    fn process(&mut self, data: &[u8]) -> Result<Consuming> {
        self.is_text &= data.iter().cloned().all(is_text);
        if self.is_text {
            Ok(Consuming::Working)
        } else {
            Ok(Consuming::Done)
        }
    }

    fn finalize(&mut self) -> Result<Selection> {
        if self.is_text {
            Ok(Selection::Select(None))
        } else {
            Ok(Selection::Ignore)
        }
    }
}

fn is_text(byte: u8) -> bool {
    (0x20..=0x7e).contains(&byte) || (0x9..=0xd).contains(&byte)
}
