use clap::{App, Arg, ArgMatches};
use std::io::prelude::*;
use std;


pub enum CriteriaArg {
    Text,
    Deflate,
}


pub struct Args {
    pub roots: Vec<String>,
    pub min_size: u64,
    pub block_size: u64,
    pub check_limit: usize,
    pub compression_ratio: f64,
    pub criteria: CriteriaArg,
    pub quiet_mode: bool,
    pub human_readable_sizes: bool,
}


pub fn args() -> Args {
    let args = parse_args();
    let criteria = match args.value_of("criteria") {
        Some("text") => CriteriaArg::Text,
        Some("deflate") => CriteriaArg::Deflate,
        Some(key) => {
            stderrln!("ERROR: Unknown value {:?} found for --criteria option.",
                      key);
            std::process::exit(1);
        }
        None => {
            stderrln!("ERROR: No value found for --criteria option.");
            std::process::exit(1);
        }
    };

    Args {
        roots: parse_strings(&args, "root"),
        min_size: parse_size(&args, "min-size"),
        block_size: parse_size(&args, "block-size"),
        check_limit: value_t_or_exit!(args, "check-limit", usize),
        compression_ratio: value_t_or_exit!(args, "compression-ratio", f64),
        criteria: criteria,
        quiet_mode: args.is_present("quiet"),
        human_readable_sizes: args.is_present("human-readable"),
    }
}


fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("big_text")
        .version("0.0.1")
        .author("Mikkel Schubert")
        .arg(Arg::with_name("quiet")
                 .short("q")
                 .long("quiet")
                 .help("Only print errors and big files."))
        .arg(Arg::with_name("human-readable")
                 .short("h")
                 .long("human-readable")
                 .help("Print sizes in a human readable format."))
        .arg(Arg::with_name("min-size")
                 .long("min-size")
                 .takes_value(true)
                 .default_value("1G")
                 .help("Minimum size of files to consider. The size is measured in \
                        bytes by default. The units 'b', 'k', 'M', 'G', 'T', and 'P', \
                        may be used to specify units of bytes, kilobytes, megabytes, \
                        gigabytes, terabytes, and petabytes"))
        .arg(Arg::with_name("block-size")
                 .long("block-size")
                 .takes_value(true)
                 // 8k is the default buffer size used by BufReader
                 .default_value("8k")
                 .help("Examine first N bytes of each file to detect text files. \
                        the same united as used by --min-size are allowed"))
        .arg(Arg::with_name("check-limit")
                 .long("check-limit")
                 .takes_value(true)
                 .default_value("10")
                 .help("If > check-limit files with an extension are found to be binary \
                        files, then subsequent files with the same extension are ignored"))
        .arg(Arg::with_name("compression-ratio")
                 .long("compression-ratio")
                 .takes_value(true)
                 .default_value("0.75")
                 .help("The highest compression ratio allowed when using the 'deflate' \
                        criteria; calcuated as new_size / old_size"))
        .arg(Arg::with_name("criteria")
                 .long("criteria")
                 .takes_value(true)
                 .default_value("text")
                 .help("The criteria used to detect candidate files; either 'text' \
                        for text files, or 'deflate' for files compressible using the \
                        deflate algorithm."))
        .arg(Arg::with_name("root")
                 .multiple(true)
                 .help("Root folder or file."))
        .get_matches()
}


fn parse_size(args: &ArgMatches, key: &str) -> u64 {
    let size = value_t_or_exit!(args, key, String);
    let (size, unit) = if let Some(index) = size.find(|c| !char::is_digit(c, 10)) {
        size.split_at(index)
    } else {
        (size.as_str(), "b")
    };

    if size.is_empty() {
        stderrln!("ERROR: No numerical value to --min-size.");
        std::process::exit(1);
    }

    let size = match u64::from_str_radix(size, 10) {
        Ok(value) => value,
        Err(err) => {
            stderrln!("ERROR: Invalid numerical passed to --min-size ({:?}): {}",
                      size,
                      err);
            std::process::exit(1);
        }
    };

    let unit = match unit {
        "b" | "B" => 1,
        "k" | "K" => 1024,
        "m" | "M" => 1024 * 1024,
        "g" | "G" => 1024 * 1024 * 1024,
        "t" | "T" => 1024 * 1024 * 1024 * 1024,
        "p" | "P" => 1024 * 1024 * 1024 * 1024 * 1024,
        _ => {
            stderrln!("Unknown unit passed to --min-size: {:?}", unit);
            std::process::exit(1);
        }
    };

    if let Some(value) = size.checked_mul(unit) {
        value
    } else {
        stderrln!("ERROR: Value passed to --min-size in bytes cannot fit in 64 bit.");
        std::process::exit(1);
    }
}


fn parse_strings(args: &ArgMatches, key: &str) -> Vec<String> {
    if let Some(values) = args.values_of(key) {
        values.map(|v| v.into()).collect()
    } else {
        vec![".".into()]
    }
}
