mod resampler;
mod features;
mod parser;
mod timing;
mod flags;

mod util;
mod dsp;
mod io;

use std::env;

use resampler::Resampler;
use parser::parse_args;

fn main() {
    println!("straycrab {}", env!("CARGO_PKG_VERSION"));

    // Check for the correct number of arguments
    if env::args().len() < 14 {
        println!("Usage: straycrab <input> <output> <pitch> <velocity> <flags> <offset> <length> <consonant> <cutoff> <volume> <modulation> <tempo> <pitchbend>");
        return;
    }

    let args: Vec<String> = env::args().collect();
    let args = parse_args(&args).expect("Failed to parse arguments");

    let resampler = Resampler::new(&args);
    resampler.render().expect("Failed to render");

    println!("Done!");
}

#[cfg(test)]
mod tests {
    use dotenv::dotenv;
    use super::*;

    #[test]
    fn test_parse_args() {
        dotenv().ok();

        let args = vec![
            "straycrab".to_string(),
            env::var("TEST_FILE").unwrap(), // input
            "test/a.wav".to_string(), // output
            "F4".to_string(), // pitch
            "60".to_string(), // velocity
            "".to_string(), // flags
            "24".to_string(), // offset
            "4000".to_string(), // length
            "56".to_string(), // consonant
            "73".to_string(), // cutoff
            "100".to_string(), // volume
            "0".to_string(), // modulation
            "120".to_string(), // tempo
            "0".to_string(), // pitchbend
        ];

        let args = parse_args(&args).expect("Failed to parse arguments");

        let resampler = Resampler::new(&args);
        resampler.render().expect("Failed to render");
    }
}
