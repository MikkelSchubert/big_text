use anyhow::{Context, Result};
use clap::{App, Arg, ArgMatches};

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

pub fn args() -> Result<Args> {
    let args = parse_args();
    let criteria = match args.value_of("criteria") {
        Some("text") => CriteriaArg::Text,
        Some("deflate") => CriteriaArg::Deflate,
        _ => unreachable!(),
    };

    Ok(Args {
        roots: args.values_of_t("root")?,
        min_size: parse_size(&args, "min-size")?,
        block_size: parse_size(&args, "block-size")?,
        check_limit: args.value_of_t("check-limit")?,
        compression_ratio: args.value_of_t("compression-ratio")?,
        criteria,
        quiet_mode: args.is_present("quiet"),
        human_readable_sizes: args.is_present("human-readable"),
    })
}

fn parse_args() -> ArgMatches {
    App::new("big_text")
        .version("0.0.1")
        .author("Mikkel Schubert")
        .arg(
            Arg::new("quiet")
                .short('q')
                .long("quiet")
                .help("Only print errors and big files."),
        )
        .arg(
            Arg::new("human-readable")
                .short('h')
                .long("human-readable")
                .help("Print sizes in a human readable format."),
        )
        .arg(
            Arg::new("min-size")
                .long("min-size")
                .takes_value(true)
                .default_value("1G")
                .help(
                    "Minimum size of files to consider. The size is measured in \
                        bytes by default. The units 'b', 'k', 'M', 'G', 'T', and 'P', \
                        may be used to specify units of bytes, kilobytes, megabytes, \
                        gigabytes, terabytes, and petabytes",
                ),
        )
        .arg(
            Arg::new("block-size")
                .long("block-size")
                .takes_value(true)
                .default_value("64k")
                .help(
                    "Examine first N bytes of each file to detect text or compressible \
                        files. The same size units used by --min-size are allowed",
                ),
        )
        .arg(
            Arg::new("check-limit")
                .long("check-limit")
                .takes_value(true)
                .default_value("10")
                .help(
                    "If > check-limit files with an extension are found to be binary \
                        files, then subsequent files with the same extension are ignored",
                ),
        )
        .arg(
            Arg::new("compression-ratio")
                .long("compression-ratio")
                .takes_value(true)
                .default_value("0.75")
                .help(
                    "The highest compression ratio allowed when using the 'deflate' \
                        criteria; calcuated as new_size / old_size",
                ),
        )
        .arg(
            Arg::new("criteria")
                .long("criteria")
                .takes_value(true)
                .possible_values(["text", "deflate"])
                .default_value("deflate")
                .help(
                    "The criteria used to detect candidate files; either 'text' \
                        for text files, or 'deflate' for files compressible using the \
                        deflate algorithm.",
                ),
        )
        .arg(
            Arg::new("root")
                .default_value(".")
                .multiple_values(true)
                .help("Root folder or file."),
        )
        .get_matches()
}

fn parse_size(args: &ArgMatches, key: &str) -> Result<u64> {
    let size: String = args.value_of_t(key)?;
    let (size, unit) = if let Some(index) = size.find(|c| !char::is_digit(c, 10)) {
        size.split_at(index)
    } else {
        (size.as_str(), "b")
    };

    let size = size
        .parse::<u64>()
        .context("invalid numerical passed to --min-size")?;

    let unit = match unit {
        "b" | "B" => 1,
        "k" | "K" => 1024,
        "m" | "M" => 1024 * 1024,
        "g" | "G" => 1024 * 1024 * 1024,
        "t" | "T" => 1024 * 1024 * 1024 * 1024,
        "p" | "P" => 1024 * 1024 * 1024 * 1024 * 1024,
        _ => {
            eprintln!("Unknown unit passed to --min-size: {:?}", unit);
            std::process::exit(1);
        }
    };

    size.checked_mul(unit)
        .context("--min-size in bytes cannot fit in 64 bit")
}
