use anyhow::{Context, Result};
use flate2::{Compress, Compression, FlushCompress, Status};

use crate::criteria::{Consuming, Criteria, Selection};

pub struct DeflatableFiles {
    deflate: Compress,
    buffer: Vec<u8>,
    ratio: f64,
}

impl DeflatableFiles {
    pub fn new(ratio: f64) -> DeflatableFiles {
        DeflatableFiles {
            deflate: Compress::new(Compression::default(), false),
            buffer: vec![0u8; 8 * 1024],
            ratio,
        }
    }
}

impl Criteria for DeflatableFiles {
    fn initialize(&mut self) {
        self.deflate.reset();
    }

    fn process(&mut self, data: &[u8]) -> Result<Consuming> {
        let in_before = self.deflate.total_in();
        let mut in_processed = 0;

        while in_processed < data.len() as u64 {
            self.deflate
                .compress(
                    &data[in_processed as usize..],
                    &mut self.buffer,
                    FlushCompress::None,
                )
                .context("error compressing file data")?;
            in_processed = self.deflate.total_in() - in_before;
        }

        Ok(Consuming::Working)
    }

    fn finalize(&mut self) -> Result<Selection> {
        loop {
            match self
                .deflate
                .compress(&[], &mut self.buffer, FlushCompress::Finish)
                .context("error finalizing deflate stream")?
            {
                Status::StreamEnd => break,
                _ => continue,
            }
        }

        let in_total = self.deflate.total_in() as f64;
        let out_total = self.deflate.total_out() as f64;
        if (out_total / in_total) <= self.ratio {
            Ok(Selection::Select)
        } else {
            Ok(Selection::Ignore)
        }
    }
}
