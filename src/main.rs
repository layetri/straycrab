mod resampler;
mod util;
mod features;
mod parser;

mod dsp;
mod io;

use std::env;

use resampler::Resampler;
use parser::parse_args;

fn main() {
    println!("straycrab {}", env!("CARGO_PKG_VERSION"));

    let args: Vec<String> = env::args().collect();
    let args = parse_args(&args).expect("Failed to parse arguments");

    let resampler = Resampler::new(&args);
    resampler.render().expect("Failed to render");

    println!("Done!");
}
