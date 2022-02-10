use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::{File, Metadata};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use walkdir::DirEntry;

use anyhow::{Context, Result};

use crate::criteria::{Consuming, Criteria, Selection, TextFiles};

#[derive(Debug)]
pub enum Checked {
    NotFile,
    TooSmall,
    Candidate(u64, PathBuf),
    Ignored,
    IgnoredExt(String),
}

pub struct FileProcessor {
    pub min_size: u64,
    pub block_size: u64,
    pub check_limit: usize,
    pub ignored_exts: HashMap<OsString, usize>,
    pub criteria: Box<dyn Criteria>,
}

impl FileProcessor {
    pub fn new() -> FileProcessor {
        FileProcessor {
            min_size: 1024 * 1024 * 1024,
            block_size: 8 * 1024,
            check_limit: 20,
            ignored_exts: HashMap::new(),
            criteria: Box::new(TextFiles::new()),
        }
    }

    pub fn set_min_size(&mut self, size: u64) {
        self.min_size = size;
    }

    pub fn set_block_size(&mut self, size: u64) {
        self.block_size = size
    }

    pub fn set_check_limit(&mut self, limit: usize) {
        self.check_limit = limit;
    }

    pub fn set_criteria(&mut self, criteria: Box<dyn Criteria>) {
        self.criteria = criteria;
    }

    pub fn process(&mut self, entry: walkdir::Result<DirEntry>) -> Result<Checked> {
        let entry = entry.context("error processing file")?;
        let file_type = entry.file_type();
        if file_type.is_symlink() || file_type.is_dir() {
            return Ok(Checked::NotFile);
        }

        let path = entry.path();
        let metadata = entry
            .metadata()
            .with_context(|| format!("error retrieving metadata for {:?}", path))?;

        if metadata.len() < self.min_size {
            return Ok(Checked::TooSmall);
        }

        if !self.ignore_extension(path) {
            let is_candidate = Self::is_candidate_file(&mut self.criteria, path, self.block_size)
                .with_context(|| format!("Error reading {:?}", path))?;

            Ok(self.update_ignored_count(path, &metadata, is_candidate))
        } else {
            Ok(Checked::Ignored)
        }
    }

    fn ignore_extension(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            self.ignored_exts.get(ext).unwrap_or(&0) > &self.check_limit
        } else {
            false
        }
    }

    fn update_ignored_count(
        &mut self,
        path: &Path,
        metadata: &Metadata,
        is_candidate: bool,
    ) -> Checked {
        if let Some(ext) = path.extension() {
            match self.ignored_exts.entry(ext.into()) {
                Entry::Occupied(mut entry) => {
                    if is_candidate {
                        entry.remove();
                    } else {
                        *entry.get_mut() += 1;
                        if entry.get() > &self.check_limit {
                            return Checked::IgnoredExt(ext.to_string_lossy().into());
                        }
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(1);
                }
            }
        }

        if is_candidate {
            Checked::Candidate(metadata.len(), path.into())
        } else {
            Checked::Ignored
        }
    }

    fn is_candidate_file(
        criteria: &mut Box<dyn Criteria>,
        path: &Path,
        mut remaining: u64,
    ) -> Result<bool> {
        let handle = File::open(path).with_context(|| format!("error opening file {:?}", path))?;
        let mut reader = BufReader::new(handle);

        criteria.initialize();
        while remaining > 0 {
            let consumed = {
                let buffer = reader
                    .fill_buf()
                    .with_context(|| format!("error reading from {:?}", path))?;
                if buffer.is_empty() {
                    break;
                }

                let to_consume = std::cmp::min(remaining, buffer.len() as u64) as usize;
                if criteria.process(&buffer[..to_consume])? != Consuming::Working {
                    break;
                }

                to_consume
            };

            remaining -= consumed as u64;
            reader.consume(consumed);
        }

        criteria.finalize().map(|v| v == Selection::Select)
    }
}
