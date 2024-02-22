use std::path::Path;

use anyhow::Result;

pub fn read_wav(file: &Path) -> Result<(Vec<f32>, u32)> {
    let mut reader = hound::WavReader::open(file)?;
    let spec = reader.spec();

    match spec.sample_format {
        hound::SampleFormat::Int => {
            let samples = reader.samples::<i32>().map(|s| s.unwrap() as f32).collect();
            Ok((samples, spec.sample_rate))
        }
        hound::SampleFormat::Float => {
            let samples = reader.samples::<f32>().map(|s| s.unwrap()).collect();
            Ok((samples, spec.sample_rate))
        }
    }
}

pub fn write_wav(file: &Path, samples: Vec<f32>, sample_rate: u32) -> Result<()> {
    std::fs::create_dir_all(file.parent().unwrap())?;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(file, spec)?;
    for sample in samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;
    use std::{env, path::PathBuf};

    #[test]
    fn test_read_write_wav() {
        let path = Path::new("test.wav");
        let samples = vec![0.0; 44100];
        let sample_rate = 44100;

        write_wav(path, samples.clone(), sample_rate).unwrap();
        let (read_samples, read_sample_rate) = read_wav(path).unwrap();

        assert_eq!(samples, read_samples);
        assert_eq!(sample_rate, read_sample_rate);
    }

    #[test]
    fn test_read_wav() {
        dotenv().ok();

        let path = PathBuf::from(env::var("TEST_FILE").unwrap());
        let (samples, sample_rate) = read_wav(&path).unwrap();
    }
}