use std::path::PathBuf;

use anyhow::Result;
use ndarray::{Array3, ArrayViewMut3};
use npyz::NpyFile;

use crate::parser::ResamplerInstruction;
use crate::features::Features;
use crate::io::audio::{read_wav, write_wav};
use crate::util::{base_frq, get_fft_size, DEFAULT_FS, F0_CEIL, F0_FLOOR};

use rsworld_sys::{HarvestOption, CheapTrickOption};


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ResamplerFlags {
    FryEnd(i32),
    FryLength(i32),
    FryOffset(i32),
    FryVolume(i32),
    FryPercentage(i32),
    VoicingTransition(i32),
    VoicingOffset(i32),
    Gender(i32),
    PitchOffset(i32),
    Tremolo(i32),
    Breathiness(i32),
    PeakCompression(i32),
    PeakNormalization(i32),
    Sibilance(i32),
    ForceFeatures
}

impl ResamplerFlags {
    pub fn from_str(s: &str) -> Option<ResamplerFlags> {
        // Destructure string into flag and value
        let parts = s.split(' ').collect::<Vec<&str>>();
        let (flag, value) = (parts[0], parts.get(1));

        match flag {
            "fe" => Some(ResamplerFlags::FryEnd(value.unwrap().parse().unwrap())),
            "fl" => Some(ResamplerFlags::FryLength(value.unwrap().parse().unwrap())),
            "fo" => Some(ResamplerFlags::FryOffset(value.unwrap().parse().unwrap())),
            "fv" => Some(ResamplerFlags::FryVolume(value.unwrap().parse().unwrap())),
            "fp" => Some(ResamplerFlags::FryPercentage(value.unwrap().parse().unwrap())),
            "ve" => Some(ResamplerFlags::VoicingTransition(value.unwrap().parse().unwrap())),
            "vo" => Some(ResamplerFlags::VoicingOffset(value.unwrap().parse().unwrap())),
            "g" => Some(ResamplerFlags::Gender(value.unwrap().parse().unwrap())),
            "t" => Some(ResamplerFlags::PitchOffset(value.unwrap().parse().unwrap())),
            "A" => Some(ResamplerFlags::Tremolo(value.unwrap().parse().unwrap())),
            "B" => Some(ResamplerFlags::Breathiness(value.unwrap().parse().unwrap())),
            "P" => Some(ResamplerFlags::PeakCompression(value.unwrap().parse().unwrap())),
            "p" => Some(ResamplerFlags::PeakNormalization(value.unwrap().parse().unwrap())),
            "S" => Some(ResamplerFlags::Sibilance(value.unwrap().parse().unwrap())),
            "G" => Some(ResamplerFlags::ForceFeatures),
            _ => None
        }
    }

    pub fn parse(s: &str) -> Vec<ResamplerFlags> {
        s.split_whitespace()
            .filter_map(ResamplerFlags::from_str)
            .collect()
    }
}

pub struct Resampler {
    pub in_file: PathBuf,
    pub out_file: PathBuf,

    pub pitch: f32,
    pub velocity: f32,
    pub flags: Vec<ResamplerFlags>,

    pub offset: f32,
    pub length: usize,
    pub consonant: f32,
    pub volume: f32,
    pub modulation: f32,
    pub tempo: f32,
    pub pitchbend: Vec<f32>,

}

impl Resampler {
    pub fn new(instruction: &ResamplerInstruction) -> Resampler {
        Resampler {
            in_file: instruction.input.clone(),
            out_file: instruction.output.clone(),
            pitch: instruction.pitch,
            velocity: instruction.velocity,
            flags: instruction.flags.clone(),
            offset: instruction.offset,
            length: instruction.length,
            consonant: instruction.consonant,
            volume: instruction.volume,
            modulation: instruction.modulation,
            tempo: instruction.tempo,
            pitchbend: instruction.pitchbend.clone(),
        }
    }

    pub fn render(&self) -> Result<()> {
        let features = self.get_features()?;

        let mut output = vec![0.0; self.length];
        let output = self.resample(output, features);

        // write_wav(&self.out_file, output, 44100)?;

        Ok(())
    }

    fn get_features(&self) -> Result<Features> {
        let path = self.in_file.with_extension("scx");

        if !self.flags.contains(&ResamplerFlags::ForceFeatures) {
            return Features::generate(&self.in_file);
        }

        if path.exists() {
            let features = Features::from_file(&path)?;

            return Ok(features);
        } else {
            return Features::generate(&self.in_file);
        }
    }

    fn resample(&self, output: Vec<f32>, features: Features) -> Result<Option<Vec<f32>>> {
        if self.out_file.to_str().unwrap() == "nul" {
            log::info!("Skipping resampling for {:?}: output file is Null", self.in_file);
            return Ok(None);
        }

        let vel = (1.0 - self.velocity / 100.0).powf(2.0);
        let vol = self.volume / 100.0;
        let modu = self.modulation / 100.0;

        log::info!("Decoding WORLD features");
        let mut sp = rsworld::decode_spectral_envelope(&features.mgc, features.mgc[0].len() as i32, DEFAULT_FS, get_fft_size());

        Ok(Some(vec![]))
    }
}