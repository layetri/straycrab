use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::io::audio::read_wav;
use crate::io::bin::{decode, encode};
use crate::util::{base_frq, F0_CEIL, F0_FLOOR};

use rsworld_sys::{HarvestOption, CheapTrickOption, D4COption};

#[derive(Debug, Serialize, Deserialize)]
pub struct FeatureDatabase {
    pub features: HashMap<String, Features>
}

impl FeatureDatabase {
    pub fn new(features: HashMap<String, Features>) -> FeatureDatabase {
        FeatureDatabase {
            features
        }
    }

    pub fn generate(file: &PathBuf) -> Result<Self> {
        // Generate features for each .wav file in the directory
        let mut lib = HashMap::new();

        for entry in tqdm::tqdm(std::fs::read_dir(file)?) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.to_str().unwrap().ends_with(".wav") {
                let features = Features::generate(&path)?;
                lib.insert(path.file_name().unwrap().to_str().unwrap().to_string(), features);
            }
        }

        let res = Self::new(lib);

        let path = file.file_stem().unwrap().to_str().unwrap();
        res.to_file(&file.join(format!("{}.scx", path)))?;

        Ok(res)
    }

    pub fn to_file(&self, path: &PathBuf) -> Result<()> {
        // let mut file = File::create(path)?;
        let serialized = bincode::serialize(&self)?;
        // file.write_all(&serialized)?;
        encode(&serialized, path)?;

        Ok(())
    }

    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let data = decode(path)?;

        // let data = std::fs::read(path)?;
        let features: FeatureDatabase = bincode::deserialize(&data)?;

        Ok(features)
    }

    pub fn get(&self, key: &str) -> Option<&Features> {
        self.features.get(key)
    }
}



#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Features {
    pub base: Vec<f64>,
    pub f0: Vec<f64>,
    pub mgc: Vec<Vec<f64>>,
    pub bap: Vec<Vec<f64>>
}

impl Features {
    pub fn new(base: Vec<f64>, f0: Vec<f64>, mgc: Vec<Vec<f64>>, bap: Vec<Vec<f64>>) -> Features {
        Features {
            base,
            f0,
            mgc,
            bap,
        }
    }
    
    pub fn to_file(&self, path: &PathBuf) -> Result<()> {
        #[cfg(feature="zstd")] {
            let serialized = bincode::serialize(&self)?;
            encode(&serialized, path)?;
        }

        #[cfg(not(feature="zstd"))] {
            let mut file = File::create(path)?;
            let serialized = bincode::serialize(&self)?;
            file.write_all(&serialized)?;
        }

        Ok(())
    }

    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let data = if cfg!(feature="zstd") {
            decode(path)?
        } else {
            std::fs::read(path)?
        };

        let features: Features = bincode::deserialize(&data)?;

        Ok(features)
    }

    pub fn generate(file: &PathBuf) -> Result<Self> {
        let mut features = Self::default();

        let (samples, sample_rate) = read_wav(file)?;
        let samples = samples.into_iter().map(|s| s as f64).collect::<Vec<f64>>();

        log::info!("Generating F0 using Harvest");
        let (f0, t) = rsworld::harvest(
            &samples, 
            sample_rate as i32,
            &HarvestOption {
                f0_floor: F0_FLOOR,
                f0_ceil: F0_CEIL,
                frame_period: 5.0
            }
        );
        let base_f0 = base_frq(&f0, Some(F0_FLOOR), Some(F0_CEIL));

        log::info!("Generating spectral envelope");
        let mut ct_option = CheapTrickOption {
            f0_floor: F0_FLOOR,
            fft_size: 512,
            q1: 0.0,
        };

        let mgc = rsworld::cheaptrick(
            &samples,
            sample_rate as i32,
            &t,
            &f0,
            &mut ct_option
        );

        log::info!("Generating aperiodicity");
        let d4c_option = D4COption {
            threshold: 0.25,
        };

        let bap = rsworld::d4c(
            &samples,
            sample_rate as i32,
            &t,
            &f0,
            &d4c_option
        );

        features.base = vec![base_f0; f0.len()];
        features.f0 = f0;
        features.mgc = mgc;
        features.bap = bap;

        features.to_file(&file.with_extension("scx"))?;

        Ok(features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::env;

    #[test]
    fn test_feature_generation() {
        dotenv().ok();

        let path = PathBuf::from(env::var("TEST_DIRECTORY").unwrap());

        let t0 = std::time::Instant::now();
        let _ = FeatureDatabase::generate(&path).unwrap();
        println!("Feature generation took: {:?}", t0.elapsed());
    }

    #[test]
    fn test_feature_reading() {
        dotenv().ok();

        let path = PathBuf::from(env::var("TEST_DIRECTORY").unwrap()).join("kasane_teto_cv.scx");

        let t0 = std::time::Instant::now();
        let features = FeatureDatabase::from_file(&path).unwrap();
        println!("Feature reading took: {:?}", t0.elapsed());

        features.get("_あ.wav");
    }

    #[test]
    fn test_generate_and_read_features() {
        dotenv().ok();

        let path = PathBuf::from(env::var("TEST_FILE").unwrap());

        let t0: std::time::Instant = std::time::Instant::now();
        let _ = Features::generate(&path).unwrap();
        println!("Feature generation took: {:?}", t0.elapsed());

        let t1 = std::time::Instant::now();
        let _ = Features::from_file(&path.with_extension("scx")).unwrap();
        println!("Feature reading took: {:?}", t1.elapsed());
    }
}