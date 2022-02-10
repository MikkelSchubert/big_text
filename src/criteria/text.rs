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

    fn process(&mut self, data: &[u8]) -> Consuming {
        self.is_text &= data.iter().cloned().all(is_text);
        if self.is_text {
            Consuming::Working
        } else {
            Consuming::Done
        }
    }

    fn finalize(&mut self) -> Selection {
        if self.is_text {
            Selection::Select
        } else {
            Selection::Ignore
        }
    }
}

fn is_text(byte: u8) -> bool {
    (0x20..=0x7e).contains(&byte) || (0x9..=0xd).contains(&byte)
}
