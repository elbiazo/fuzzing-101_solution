mod parser;
use libafl_sugar::ForkserverBytesCoverageSugar;
pub const MAP_SIZE: usize = 80642;

fn main() {
    let parsed_opts = parser::parse_args();

    // ./build/exercise-3 -i corpus/ -o solutions/ -c 0-7 -t ./build/sbin/tcpdump --args -vr @@
    ForkserverBytesCoverageSugar::<MAP_SIZE>::builder()
        .input_dirs(&[parsed_opts.input])
        .output_dir(parsed_opts.output)
        .cores(&parsed_opts.cores)
        .program(parsed_opts.target)
        .arguments(&parsed_opts.args)
        .build()
        .run()
}
