use num_traits::Pow;

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct ResamplerFlags {
    pub fry_end: Option<f64>,
    pub fry_length: Option<f64>,
    pub fry_offset: Option<f64>,
    pub fry_volume: Option<i32>,
    pub fry_pitch: Option<f64>,
    pub voicing_transition: Option<i32>,
    pub voicing_offset: Option<i32>,
    pub gender: Option<f64>,
    pub pitch_offset: Option<i32>,
    pub tremolo: Option<i32>,
    pub breathiness: Option<i32>,
    pub peak_compression: Option<i32>,
    pub peak_normalization: Option<i32>,
    pub sibilance: Option<i32>,
    pub force_features: bool
}

impl ResamplerFlags {
    pub fn parse(flags: &str) -> ResamplerFlags {
        let mut res = ResamplerFlags::default();

        for f in flags.split("|").collect::<Vec<&str>>() {
            let parts = f.split(' ').collect::<Vec<&str>>();
            let (flag, value) = (parts[0], parts.get(1));

            let value: i32 = value.map(|v| v.parse().unwrap()).unwrap_or(0);

            match flag {
                "fe" => res.fry_end = Some(value as f64 / 1000.0),
                "fl" => res.fry_length = Some((value as f64 / 1000.0).max(0.001)),
                "fo" => res.fry_offset = Some(value as f64 / 1000.0),
                "fv" => res.fry_volume = Some(value),
                "fp" => res.fry_pitch = Some(value.max(0) as f64),
                "ve" => res.voicing_transition = Some(value),
                "vo" => res.voicing_offset = Some(value),
                "g" => res.gender = Some((value as f64 / 120.0).pow(2.0)),
                "B" => res.breathiness = Some(value),
                "P" => res.peak_compression = Some(value),
                "p" => res.peak_normalization = Some(value),
                "A" => res.tremolo = Some(value),
                "t" => res.pitch_offset = Some(value),
                "S" => res.sibilance = Some(value),
                "G" => res.force_features = true,
                _ => {}
            }
        }

        res
    }
}