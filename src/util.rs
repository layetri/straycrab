use std::collections::HashMap;

use anyhow::Result;

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

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    3.0 * t.powi(2) - 2.0 * t.powi(3)
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

pub fn to_int12_stream(data: Vec<&str>) -> Vec<i64> {
    let mut res = vec![];

    for i in (0..data.len()).step_by(2) {
        // Convert base64 to list of integers
        let b = data[i..i+2].to_vec().join("");
        let b = i64::from_str_radix(&b, 16).unwrap();
        res.push(b);
    }

    res
}

pub fn pitch_string_to_cents(s: &String) -> Result<Vec<f32>> {
    let pitch: Vec<&str> = s.split("#").collect();
    let mut res = vec![];

    for i in (0..pitch.len()).step_by(2) {
        let p = pitch[i..i+2].to_vec();

        // if p.len() == 2 {
        //     let (pitch_str, rle) = (p[0], p[1]);
        //     res.extend(to_int12_stream(pitch_str));
        //     res.extend([res[res.len()-1]] * rle.parse::<usize>()?);
        // } else {
        //     res.extend(to_int12_stream(p[0]));
        // }
    }

    Ok(res)
}