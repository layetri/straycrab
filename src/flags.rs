#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct ResamplerFlags {
    pub fry_end: Option<i32>,
    pub fry_length: Option<i32>,
    pub fry_offset: Option<i32>,
    pub fry_volume: Option<i32>,
    pub fry_pitch: Option<i32>,
    pub voicing_transition: Option<i32>,
    pub voicing_offset: Option<i32>,
    pub gender: Option<i32>,
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

            match flag {
                "fe" => res.fry_end = value.map(|v| v.parse().unwrap()),
                "fl" => res.fry_length = value.map(|v| v.parse().unwrap()),
                "fo" => res.fry_offset = value.map(|v| v.parse().unwrap()),
                "fv" => res.fry_volume = value.map(|v| v.parse().unwrap()),
                "fp" => res.fry_pitch = value.map(|v| v.parse().unwrap()),
                "ve" => res.voicing_transition = value.map(|v| v.parse().unwrap()),
                "vo" => res.voicing_offset = value.map(|v| v.parse().unwrap()),
                "g" => res.gender = value.map(|v| v.parse().unwrap()),
                "B" => res.breathiness = value.map(|v| v.parse().unwrap()),
                "P" => res.peak_compression = value.map(|v| v.parse().unwrap()),
                "p" => res.peak_normalization = value.map(|v| v.parse().unwrap()),
                "A" => res.tremolo = value.map(|v| v.parse().unwrap()),
                "t" => res.pitch_offset = value.map(|v| v.parse().unwrap()),
                "S" => res.sibilance = value.map(|v| v.parse().unwrap()),
                "G" => res.force_features = true,
                _ => {}
            }
        }

        res
    }
}