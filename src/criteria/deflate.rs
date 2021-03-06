use flate2::{Compress, Compression, Flush, Status};

use criteria::{Criteria, Consuming, Selection};


pub struct DeflatableFiles {
    deflate: Compress,
    buffer: Vec<u8>,
    ratio: f64,
}


impl DeflatableFiles {
    pub fn new(ratio: f64) -> DeflatableFiles {
        DeflatableFiles {
            deflate: Compress::new(Compression::Default, false),
            buffer: vec![0u8; 8 * 1024],
            ratio: ratio,
        }
    }
}


impl Criteria for DeflatableFiles {
    fn initialize(&mut self) {
        self.deflate.reset();
    }

    fn process(&mut self, data: &[u8]) -> Consuming {
        let in_before = self.deflate.total_in();
        let mut in_processed = 0;

        while in_processed < data.len() as u64 {
            self.deflate
                .compress(&data[in_processed as usize..],
                          &mut self.buffer,
                          Flush::None);
            in_processed = self.deflate.total_in() - in_before;
        }

        Consuming::Working
    }

    fn finalize(&mut self) -> Selection {
        loop {
            match self.deflate
                      .compress(&[], &mut self.buffer, Flush::Finish) {
                Status::StreamEnd => break,
                _ => continue,
            }
        }

        let in_total = self.deflate.total_in() as f64;
        let out_total = self.deflate.total_out() as f64;
        if (out_total / in_total) <= self.ratio {
            Selection::Select
        } else {
            Selection::Ignore
        }
    }
}
