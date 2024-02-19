use std::{fs::File, io::{BufRead, BufReader, Read, Write}, path::PathBuf};

use anyhow::Result;

pub fn encode(data: &[u8], path: &PathBuf) -> Result<()> {
    // Encode data to zstd compressed file
    let file = File::create(path)?;
    let mut encoder = zstd::Encoder::new(file, 0)?;

    encoder.write_all(data)?;
    encoder.finish()?;

    Ok(())
}

pub fn decode(path: &PathBuf) -> Result<Vec<u8>> {
    // Decode zstd compressed file
    let mut file = File::open(path)?;
    let decoder = zstd::Decoder::new(file)?;

    let mut decoded: Vec<u8> = vec![];
    let bytes = BufReader::new(decoder).bytes();
    for byte in bytes {
        decoded.push(byte?);
    }

    Ok(decoded)
}


#[cfg(test)]
mod tests {
    use crate::features::Features;

    use super::*;

    #[test]
    fn test_encode_decode() {
        let path = PathBuf::from("test.zst");
        let data = Features::default();
        
        // Pickle the data
        let t0 = std::time::Instant::now();
        let serialized = bincode::serialize(&data).unwrap();
        println!("Serialization took: {:?}", t0.elapsed());

        let t1 = std::time::Instant::now();
        encode(&serialized, &path).unwrap();
        println!("Encoding took: {:?}", t1.elapsed());

        let t2 = std::time::Instant::now();
        let decoded = decode(&path).unwrap();
        println!("Decoding took: {:?}", t2.elapsed());

        assert_eq!(serialized, decoded.as_slice());

        // Unpickle the data
        let t3 = std::time::Instant::now();
        let deserialized: Features = bincode::deserialize(&decoded).unwrap();
        println!("Deserialization took: {:?}", t3.elapsed());

        assert_eq!(data, deserialized);
    }
}