use clap::{App, Arg};
use libafl::bolts::os::Cores;
use std::path::PathBuf;

pub struct FuzzerOptions {
    pub output: PathBuf,
    pub input: PathBuf,
    pub cores: Cores,
    pub target: String,
    pub args: Vec<String>,
}

pub fn parse_args() -> FuzzerOptions {
    let matches = App::new("Fuzzer Options")
        .version("0.1")
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("DIR")
                .help("Sets the output directory to DIR")
                .takes_value(true),
        )
        .arg(
            Arg::new("input")
                .multiple_values(true)
                .short('i')
                .long("input")
                .value_name("DIR")
                .help("Sets the input directory to DIR")
                .takes_value(true),
        )
        .arg(
            Arg::new("cores")
                .short('c')
                .long("cores")
                .value_name("CORES")
                .help("Sets the number of cores to use")
                .takes_value(true),
        )
        .arg(
            Arg::new("target")
                .short('t')
                .long("target")
                .value_name("PROGRAM")
                .help("Sets the target program to PROGRAM")
                .takes_value(true),
        )
        .arg(
            Arg::new("args")
                .multiple_values(true)
                .allow_hyphen_values(true)
                .short('a')
                .long("args")
                .value_name("ARGS")
                .help("Sets the arguments to PROGRAM")
                .takes_value(true),
        )
        .get_matches();

    FuzzerOptions {
        output: matches.value_of("output").unwrap_or("").into(),
        input: matches.value_of("input").unwrap_or("").into(),
        cores: Cores::from_cmdline(matches.value_of("cores").unwrap_or("").into()).unwrap(),
        target: matches.value_of("target").unwrap_or("").into(),
        args: matches
            .values_of("args")
            .unwrap()
            .map(|s| s.into())
            .collect(),
    }
}
