use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{File, Metadata};
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std;
use walkdir::DirEntry;
use walkdir;

use error::ProcError;


#[derive(Debug)]
pub enum Checked {
    NotFile,
    TooSmall,
    BigText(u64, PathBuf),
    Binary,
    NewBinaryExt(String),
}


pub struct FileProcessor {
    pub min_size: u64,
    pub block_size: u64,
    pub check_limit: usize,
    pub excluded_exts: HashMap<OsString, usize>,
}


impl Default for FileProcessor {
    fn default() -> FileProcessor {
        FileProcessor {
            min_size: 1024 * 1024 * 1024,
            block_size: 5 * 1024,
            check_limit: 20,
            excluded_exts: HashMap::new(),
        }
    }
}


impl FileProcessor {
    pub fn process(&mut self, entry: walkdir::Result<DirEntry>) -> Result<Checked, ProcError> {
        let entry = entry
            .map_err(|e| ProcError::new("Error processing file", e))?;
        let file_type = entry.file_type();
        if file_type.is_symlink() || file_type.is_dir() {
            return Ok(Checked::NotFile);
        }

        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|e| {
                         let msg = format!("Error retrieving metadata for {:?}", path);
                         ProcError::new(&msg, e)
                     })?;

        if metadata.len() < self.min_size {
            return Ok(Checked::TooSmall);
        }

        if !self.ignore_extension(path) {
            let is_text =
                Self::is_text_file(path, self.block_size)
                    .map_err(|e| ProcError::new(&format!("Error reading {:?}", path), e))?;

            Ok(self.update_ignored_count(path, &metadata, is_text))
        } else {
            Ok(Checked::Binary)
        }
    }

    fn ignore_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            self.excluded_exts.get(ext.into()).unwrap_or(&0) > &self.check_limit
        } else {
            false
        }
    }

    fn update_ignored_count(&mut self, path: &Path, metadata: &Metadata, is_text: bool) -> Checked {
        if let Some(ext) = path.extension() {
            match self.excluded_exts.entry(ext.into()) {
                Entry::Occupied(mut entry) => {
                    if is_text {
                        entry.remove();
                    } else {
                        *entry.get_mut() += 1;
                        if entry.get() > &self.check_limit {
                            return Checked::NewBinaryExt(ext.to_string_lossy().into());
                        }
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(1);
                }
            }
        }

        if is_text {
            Checked::BigText(metadata.len(), path.into())
        } else {
            Checked::Binary
        }
    }

    fn is_text_file(path: &Path, mut block_size: u64) -> std::io::Result<bool> {
        let handle = File::open(path)?;
        let mut reader = BufReader::new(handle);

        while block_size > 0 {
            let consumed = {
                let buffer = reader.fill_buf()?;
                if buffer.is_empty() {
                    break;
                }

                let to_consume = std::cmp::min(block_size, buffer.len() as u64) as usize;
                if !buffer[..to_consume].iter().cloned().all(Self::is_text) {
                    return Ok(false);
                }

                to_consume
            };

            block_size -= consumed as u64;
            reader.consume(consumed);
        }

        Ok(true)
    }

    fn is_text(byte: u8) -> bool {
        (byte >= 0x20 && byte <= 0x7e) || (byte >= 0x9 && byte <= 0xd)
    }
}
