extern crate clap;
extern crate flate2;
extern crate walkdir;

use args::CriteriaArg;
use walkdir::WalkDir;

mod args;
mod criteria;
mod processor;

use criteria::{Criteria, DeflatableFiles, TextFiles};
use processor::{Checked, FileProcessor};

fn human_readable_size(n_bytes: u64) -> String {
    let (div, desc) = match n_bytes {
        0..=1023 => return format!("{}", n_bytes),
        1024..=1048575 => (2u64.pow(10), " KB"),
        1048576..=1073741823 => (2u64.pow(20), " MB"),
        1073741824..=1099511627775 => (2u64.pow(30), " GB"),
        _ => (2u64.pow(40), " TB"),
    };

    format!("{:.1}{}", n_bytes as f64 / div as f64, desc)
}

fn machine_readable_size(n_bytes: u64) -> String {
    format!("{}", n_bytes)
}

fn eprint_error(error: anyhow::Error) {
    eprint!("ERR: {}", error);

    let mut cause = error.source();
    while let Some(err) = cause {
        eprint!(", caused by {}", err);
        cause = err.source();
    }

    eprintln!();
}

fn create_processor(args: &args::Args) -> processor::FileProcessor {
    let mut processor = FileProcessor::new();
    processor.set_min_size(args.min_size);
    processor.set_block_size(args.block_size);
    processor.set_check_limit(args.check_limit);

    let criteria = match args.criteria {
        args::CriteriaArg::Text => Box::new(TextFiles::new()) as Box<dyn Criteria>,
        args::CriteriaArg::Deflate => {
            Box::new(DeflatableFiles::new(args.compression_ratio)) as Box<dyn Criteria>
        }
    };

    processor.set_criteria(criteria);
    processor
}

fn main() -> Result<(), anyhow::Error> {
    let args = args::args()?;
    let mut processor = create_processor(&args);

    let mut errors = 0;
    let mut total_size = 0;
    let mut total_size_cmp = 0;
    let mut total_checked = 0;
    let mut non_files_skipped = 0;
    let mut small_files_skipped = 0;
    let mut files_skipped = 0;
    let mut candidates_found = 0;

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
                    eprint_error(error);
                    errors += 1;
                }
                Ok(Checked::NotFile) => non_files_skipped += 1,
                Ok(Checked::TooSmall) => small_files_skipped += 1,
                Ok(Checked::Ignored) => files_skipped += 1,
                Ok(Checked::IgnoredExt(ext)) => {
                    files_skipped += 1;
                    if !args.quiet_mode {
                        eprintln!("Now skipping files with extension *.{}", ext);
                    }
                }
                Ok(Checked::Candidate(size, ratio, path)) => {
                    println!("{}\t{}", format_size(size), path.to_string_lossy());
                    total_size += size;
                    total_size_cmp += (size as f64 * ratio.unwrap_or_default()) as u64;
                    candidates_found += 1;
                }
            }
        }
    }

    if !args.quiet_mode {
        eprintln!("Files checked = {}", total_checked);
        eprintln!("Small files skipped = {}", small_files_skipped);
        eprintln!("Non-files skipped = {}", non_files_skipped);
        eprintln!("Files ignored = {}", files_skipped);
        eprintln!("Candidate files found = {}", candidates_found);
        eprintln!("Size of candidate files = {}", format_size(total_size));
        if matches!(args.criteria, CriteriaArg::Deflate) {
            eprintln!(
                "Est. size saved by compression = {}",
                format_size(total_size - total_size_cmp)
            );
        }
        eprintln!("Errors encountered = {}", errors);
    }

    Ok(())
}
