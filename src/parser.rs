use std::path::PathBuf;

use anyhow::Result;

use crate::flags::ResamplerFlags;
use crate::util::misc::{note_to_midi, pitch_string_to_cents};

#[derive(Debug, Default)]
pub struct ResamplerInstruction {
    pub input: PathBuf,
    pub output: PathBuf,
    pub pitch: f64,
    pub velocity: f32,
    pub flags: ResamplerFlags,
    pub offset: f32,
    pub length: usize,
    pub consonant: f32,
    pub cutoff: f32,
    pub volume: f32,
    pub modulation: f64,
    pub tempo: f32,
    pub pitchbend: Vec<i16>,
}

pub fn parse_args(args: &Vec<String>) -> Result<ResamplerInstruction> {
    Ok(ResamplerInstruction {
        input: PathBuf::from(&args[1]),
        output: PathBuf::from(&args[2]),
        pitch: note_to_midi(args[3].as_str()),
        velocity: args[4].parse::<f32>()?,
        flags: ResamplerFlags::parse(&args[5]),
        offset: args[6].parse::<f32>()?,
        length: args[7].parse::<usize>()?,
        consonant: args[8].parse::<f32>()?,
        cutoff: args[9].parse::<f32>()?,
        volume: args[10].parse::<f32>()?,
        modulation: (args[11].parse::<f64>()?) / 100.0,
        tempo: args[12].parse::<f32>()?,
        pitchbend: pitch_string_to_cents(&args[13])?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let args = vec![
            "resampler".to_string(),
            "input.wav".to_string(),
            "output.wav".to_string(),
            "100".to_string(),
            "100".to_string(),
            "G 0.0 0".to_string(),
            "0.0".to_string(),
            "0".to_string(),
            "0.0".to_string(),
            "0.0".to_string(),
            "0.0".to_string(),
            "0.0".to_string(),
            "0.0".to_string(),
        ];

        let resampler = parse_args(&args).unwrap();
        
        assert_eq!(resampler.input, PathBuf::from("input.wav"));
        assert_eq!(resampler.output, PathBuf::from("output.wav"));
    }
}