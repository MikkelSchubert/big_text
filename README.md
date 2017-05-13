Locate text files or uncompressed files, to help archving where the original file-structure should be retained as much as possible.

By default the program looks for big text files, at least 1GB in size. To identify text files, the first N bytes are checked, defaulting to the first 8 kb. Alternatively, the compression ratio obtained using deflate on the first N bytes can be used to identify files that can be compressed for a given gain. By default, the program will report files with a compressed to uncompressed ratio of 0.75 or less.

Non-text or non-compressible files are automatically classified by their extension. If 10 or more files in a row (for a given extension) fail the text or deflate check, then subsequent files with that extension are ignored.

## Example output

    $ big\_text -h
    1.3 GB    ./path/to/big/file.tsv
    10.1 GB   ./path/to/bigger/file.mpileup
    [...]
    Files checked = 577
     - Small files skipped = 402
     - Non-files skipped = 175
     - Ignored files = 0
    Candidate files found = 17
     - Total size = 291.3 GB
    Errors encountered = 0

## Usage

    USAGE:
        big_text [FLAGS] [OPTIONS] [root]...

    FLAGS:
            --help              Prints help information
        -h, --human-readable    Print sizes in a human readable format.
        -q, --quiet             Only print errors and big files.
        -V, --version           Prints version information

    OPTIONS:
            --block-size <block-size>
                Examine first N bytes of each file to detect text or
                compressible files. The same size units used by --min-size
                are allowed [default: 8k]
            --check-limit <check-limit>
                If > check-limit files with an extension are found to be binary
                files, then subsequent files with the same extension are
                ignored [default: 10]
            --compression-ratio <compression-ratio>
                The highest compression ratio allowed when using the 'deflate'
                criteria; calcuated as new_size / old_size [default: 0.75]
            --criteria <criteria>
                The criteria used to detect candidate files; either 'text' for
                text files, or 'deflate' for files compressible using the
                deflate algorithm. [default: text]
            --min-size <min-size>
                Minimum size of files to consider. The size is measured in
                bytes by default. The units 'b', 'k', 'M', 'G', 'T', and 'P',
                may be used to specify units of bytes, kilobytes, megabytes,
                gigabytes, terabytes, and petabytes [default: 1G]

    ARGS:
        <root>...    Root folder or file.
