use ndarray::prelude::*;

use std::collections::HashMap;
use anyhow::Result;
use regex::Regex;

pub const F0_FLOOR: f64 = 71.0;
pub const F0_CEIL: f64 = 1760.0;
pub const DEFAULT_FS: i32 = 44100;


pub fn get_notes() -> HashMap<String, i32> {
    HashMap::from([
        ("C".to_string(), 0),
        ("C#".to_string(), 1),
        ("D".to_string(), 2),
        ("D#".to_string(), 3),
        ("E".to_string(), 4),
        ("F".to_string(), 5),
        ("F#".to_string(), 6),
        ("G".to_string(), 7),
        ("G#".to_string(), 8),
        ("A".to_string(), 9),
        ("A#".to_string(), 10),
        ("B".to_string(), 11)
    ])
}

pub fn get_fft_size() -> i32 {
    2048
}

pub fn smoothstep(edge0: f64, edge1: f64, x: &Array1<f64>) -> Array1<f64> {
    let t = ((x - edge0) / (edge1 - edge0));
    let t = t.mapv(|v| v.clamp(0.0, 1.0));

    3.0 * &t*&t - 2.0 * &t*&t*&t
}

pub fn bias(x: f32, a: f32) -> f32 {
    x / ((1.0 / a - 2.0) * (1.0 - x) + 1.0)
}

pub fn base_frq(f0: &Vec<f64>, f0_min: Option<f64>, f0_max: Option<f64>) -> f64 {
    let mut q = 0.0;
    let mut avg_frq = 0.0;
    let mut weight = 0.0;
    let mut tally = 0.0;

    let mut N = f0.len();

    let f0_min = f0_min.unwrap_or(F0_FLOOR);
    let f0_max = f0_max.unwrap_or(F0_CEIL);

    for i in 0..N {
        if f0[i] > f0_min && f0[i] < f0_max {
            if i < 1 {
                q = f0[i + 1] - f0[i];
            } else if i == N - 1 {
                q = f0[i] - f0[i - 1];
            } else {
                q = (f0[i + 1] - f0[i - 1]) / 2.0;
            }

            weight = 2.0f64.powf(-q * q);
            avg_frq += f0[i] * weight;
            tally += weight;
        }
    }

    if tally > 0.0 {
        avg_frq / tally
    } else {
        avg_frq
    }
}

pub fn to_uint6(data: &str) -> u8 {
    let x = *data.bytes().collect::<Vec<u8>>().first().unwrap();

    if x >= 97 {
        x - 71
    } else if x >= 65 {
        x - 65
    } else if x >= 48  {
        x + 4
    } else if x == 43 {
        62
    } else if x == 47 {
        63
    } else {
        0
    }
}

pub fn to_int12(data: (char, char)) -> i16 {
    let uint12: i16 = (to_uint6(&data.0.to_string()) as i16) << 6 | to_uint6(&data.1.to_string()) as i16;

    if uint12 >> 11 & 1 == 1 {
        uint12 - (1 << 12)
    } else {
        uint12
    }
}

pub fn to_int12_stream(data: &str) -> Vec<i16> {
    let mut res = vec![];

    for i in (0..data.len()).step_by(2) {
        // Convert base64 to list of integers
        let b = &data[i..i+2];
        res.push(to_int12((b.chars().nth(0).unwrap(), b.chars().nth(1).unwrap())));
    }

    res
}

pub fn pitch_string_to_cents(s: &str) -> Result<Vec<i16>> {
    if s.len() < 2 {
        return Ok(vec![0]);
    }

    let pitch: Vec<&str> = s.split("#").collect();
    let mut res = vec![];

    for i in (0..pitch.len()).step_by(2) {
        if i+2 < pitch.len() {
            let p = pitch[i..i+2].to_vec();

            let (pitch_str, rle) = (p[0], p[1]);
            res.extend(to_int12_stream(pitch_str));
            res.extend(vec![res[res.len()-1]; rle.parse::<usize>().unwrap()]);
        } else {
            res.extend(to_int12_stream(pitch[i]));
        }
    }

    res.push(0);

    Ok(res)
}

pub fn mtof(pitch: f64) -> f64 {
    440.0 * 2.0f64.powf((pitch - 69.0) / 12.0)
}

pub fn ftom(freq: f64) -> f64 {
    69.0 + 12.0 * (freq / 440.0).log2()
}

pub fn note_to_midi(note: &str) -> f64 {
    // Regex to split note into note and octave
    let re = Regex::new(r"([A-Ga-g]+)(\d+)").unwrap();
    let caps = re.captures(note).unwrap();

    let note = caps.get(1).unwrap().as_str();
    let octave = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();

    let notes = get_notes();
    let note = notes.get(note).unwrap();

    (12 * (octave + 1) + *note) as f64
}

pub fn dump_test_data(sp: &Vec<Vec<f64>>, ap: &Vec<Vec<f64>>, f0: &Vec<f64>, t: &Vec<f64>, path: &str) {
    let mut data = String::new();
    let j = sp[0].len() / 2;

    data.push_str("[Time]\t[F0]\t[Harmonic]\t[Aperiodic]\n");

    for i in 0..sp.len() {
        data.push_str(&format!("{}\t{}\t{}\t{}\n", t[i], f0[i], sp[i][0], ap[i][0]));
    }

    std::fs::create_dir_all("test").unwrap();
    std::fs::write(path, data).unwrap();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pitch_string_to_cents() {
        let s = "B7CPCVCVCTCQCNCICDB+B5B0BvBrBnBlBk#14#BjBF/++Y8k615d4p4f4l4y5G5f596e7B7l8H8n9D9Z9q9092919y9t9n9f9Y9Q9I9C898584858/9L9b9v+G+f+4/Q/m/5AIATAY#2#AWAUARAOALAHAFACABAA";

        let res = pitch_string_to_cents(&s).unwrap();

        println!("{:?}", res);
    }
}