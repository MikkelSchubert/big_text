macro_rules! stderr(
    ($($arg:tt)*) => { {
        let result = write!(&mut ::std::io::stderr(), $($arg)*);
        result.expect("error printing to stderr");
    } }
);

macro_rules! stderrln(
    ($($arg:tt)*) => { {
        let result = writeln!(&mut ::std::io::stderr(), $($arg)*);
        result.expect("error printing to stderr");
    } }
);
