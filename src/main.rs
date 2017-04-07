#[macro_use(value_t_or_exit)]
extern crate clap;
extern crate walkdir;

use std::io::Write;
use walkdir::WalkDir;

#[macro_use]
mod stderr;

mod args;
mod error;
mod processor;

use processor::{Checked, FileProcessor};


fn human_readable_size(n_bytes: u64) -> String {
    let (div, desc) = match n_bytes {
        0...1023 => return format!("{}", n_bytes),
        1024...1048575 => (2u64.pow(10), " KB"),
        1048576...1073741823 => (2u64.pow(20), " MB"),
        1073741824...1099511627775 => (2u64.pow(30), " GB"),
        _ => (2u64.pow(40), " TB"),
    };

    format!("{:.1}{}", n_bytes as f64 / div as f64, desc)
}


fn machine_readable_size(n_bytes: u64) -> String {
    format!("{}", n_bytes)
}



fn main() {
    let args = args::args();
    let mut processor = FileProcessor {
        min_size: args.min_size,
        block_size: args.block_size,
        check_limit: args.check_limit,
        ..FileProcessor::default()
    };

    let mut errors = 0;
    let mut total_size = 0;
    let mut total_checked = 0;
    let mut non_files_skipped = 0;
    let mut small_files_skipped = 0;
    let mut binary_files_skipped = 0;
    let mut big_text_files_found = 0;

    let format_size = if args.human_readable_sizes {
        human_readable_size
    } else {
        machine_readable_size
    };

    for path in args.roots {
        let walker = WalkDir::new(path);
        for entry in walker {
            total_checked += 1;

            match processor.process(entry) {
                Err(error) => {
                    stderrln!("{}", error);
                    errors += 1;
                }
                Ok(Checked::NotFile) => non_files_skipped += 1,
                Ok(Checked::TooSmall) => small_files_skipped += 1,
                Ok(Checked::NewBinaryExt(ext)) => {
                    binary_files_skipped += 1;
                    if !args.quiet_mode {
                        stderrln!("Now skipping files with extension *.{}", ext);
                    }
                }
                Ok(Checked::Binary) => binary_files_skipped += 1,
                Ok(Checked::BigText(size, path)) => {
                    println!("{}\t{}", format_size(size), path.to_string_lossy());
                    total_size += size;
                    big_text_files_found += 1;
                }
            }
        }
    }

    if !args.quiet_mode {
        stderrln!("Files checked = {}", total_checked);
        stderrln!(" - Small files skipped = {}", small_files_skipped);
        stderrln!(" - Binary files skipped = {}", binary_files_skipped);
        stderrln!(" - Non-files skipped = {}", non_files_skipped);
        stderrln!("Big text files found = {}", big_text_files_found);
        stderrln!(" - Total size = {}", format_size(total_size));
        stderrln!("Errors encountered = {}", errors);
    }
}
