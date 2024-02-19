use std::path::PathBuf;

use anyhow::Result;
use csaps::CubicSmoothingSpline;
use makima_spline::{vec_to_points, Spline};
use ndarray::Array1;
use num_traits::Pow;
use num_traits::real::Real;

use crate::parser::ResamplerInstruction;
use crate::features::Features;
use crate::io::audio::{read_wav, write_wav};
use crate::timing::TimingData;
use crate::util::misc::{get_fft_size, mtof, smoothstep, DEFAULT_FS, F0_FLOOR};
use crate::util::math::{linspace, Scalar};
use crate::flags::ResamplerFlags;


pub struct Resampler {
    pub in_file: PathBuf,
    pub out_file: PathBuf,

    pub pitch: f64,
    pub velocity: f32,
    pub flags: ResamplerFlags,

    pub offset: f32,
    pub length: usize,
    pub consonant: f32,
    pub cutoff: f32,
    pub volume: f32,
    pub modulation: f32,
    pub tempo: f32,
    pub pitchbend: Vec<f64>,

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
            cutoff: instruction.cutoff,
            volume: instruction.volume,
            modulation: instruction.modulation,
            tempo: instruction.tempo,
            pitchbend: instruction.pitchbend.clone(),
        }
    }

    pub fn render(&self) -> Result<()> {
        let features = self.get_features()?;
        let output = self.resample(features)?;

        if let Some(output) = output {
            write_wav(&self.out_file, output, 44100)?;
        }

        Ok(())
    }

    fn get_features(&self) -> Result<Features> {
        let path = self.in_file.with_extension("scx");

        if !self.flags.force_features {
            return Features::generate(&self.in_file);
        }

        if path.exists() {
            let features = Features::from_file(&path)?;

            Ok(features)
        } else {
            Features::generate(&self.in_file)
        }
    }

    fn resample(&self, features: Features) -> Result<Option<Vec<f32>>> {
        if self.out_file.to_str().unwrap() == "nul" {
            log::info!("Skipping resampling for {:?}: output file is Null", self.in_file);
            return Ok(None);
        }

        
        let vol = self.volume / 100.0;
        let modu = self.modulation as f64 / 100.0;

        log::info!("Decoding WORLD features");
        let mut sp = rsworld::decode_spectral_envelope(&features.mgc, features.mgc[0].len() as i32, DEFAULT_FS, get_fft_size());
        let mut ap = rsworld::decode_aperiodicity(&features.bap, features.bap[0].len() as i32, DEFAULT_FS);

        // Generate F0 offsets relative to base frequency
        let mut f0_off = vec![];
        for f in &features.f0 {
            if f > &0.0 {
                f0_off.push(f - features.base);
            } else {
                f0_off.push(features.base);
            }
        }

        let timing = TimingData::calculate(features.f0.len(), self.offset, self.cutoff, self.consonant);

        let t_render = self.interpolate_features(&mut sp, &mut ap, &mut f0_off, &timing)?;
        let t = linspace(0.0, f0_off.len() as f64 * 0.005, f0_off.len());


        // Generate pitch parameters
        let pitch: Vec<f64> = self.pitchbend.iter().map(|x| x / 100.0 + self.pitch).collect();

        let mut t_pitch: Vec<f64> = vec![];
        for i in 0..pitch.len() {
            t_pitch.push((60.0 * i as f64) / (self.tempo as f64 * 96.0));
        }

        let mut pitch_render = vec![];
        let pitch_interpolator = Spline::from_vec(vec_to_points(&t_pitch, &pitch));        
        for t in t_pitch {
            pitch_render.push(pitch_interpolator.sample(t));
        }


        // Check if flags has the PitchOffset flag, if so, apply its value to the pitch
        if let Some(offset) = self.flags.pitch_offset {
            for i in 0..pitch_render.len() {
                pitch_render[i] += offset as f64;
            }
        }


        let mut f0 = vec![];
        for i in 0..features.f0.len() {
            f0.push(mtof(pitch_render[i]) + f0_off[i] * modu);
        }

        // Process pre-render flags
        self.process_prerender_flags(&mut sp, &mut ap, &mut f0, &t, &timing)?;

        // Yass, slay, synthesize
        let render = rsworld::synthesis(&f0, &sp, &ap, 5.0, DEFAULT_FS);

        // Apply post-render flags
        // TODO: Implement post-render flags


        let render = render.iter().map(|x| *x as f32).collect::<Vec<f32>>();
        Ok(Some(render))
    }
    

    fn interpolate_features(&self, sp: &mut Vec<Vec<f64>>, ap: &mut Vec<Vec<f64>>, f0_off: &mut Vec<f64>, timing: &TimingData) -> Result<Vec<f64>> {
        let f0_off_interpolator = CubicSmoothingSpline::new(&timing.positions, &f0_off).make().unwrap();

        let vel = (1.0 - self.velocity / 100.0).powf(2.0);
        let length_req = self.length as f32 / 1000.0;
        let stretch_length = timing.end - timing.start;

        // Generate timing vectors for consonant and stretch areas
        let t_consonant = linspace(timing.start as f64, timing.con as f64, (vel * self.consonant / 5.0) as usize);
        let t_stretch = if stretch_length > length_req {
            let con_idx = (200.0 * timing.con) as usize;
            let len_idx = (200.0 * length_req) as usize;

            timing.positions[con_idx..len_idx].to_vec()
        } else {
            linspace(timing.con as f64, timing.end as f64, (200.0 * length_req) as usize)
        };

        let t_render = [t_consonant, t_stretch].concat();
        let mut new_sp = vec![vec![0.0; sp[0].len()]; t_render.len()];
        let mut new_ap = vec![vec![0.0; ap[0].len()]; t_render.len()];
        
        for i in 0..sp.len() {
            let sp_points = vec_to_points(&timing.positions, &sp[i]);
            let ap_points = vec_to_points(&timing.positions, &ap[i]);

            let sp_interpolator = Spline::from_vec(sp_points);
            let ap_interpolator = Spline::from_vec(ap_points);

            for j in 0..t_render.len() {
                let t = t_render[j];

                let _sp = sp_interpolator.sample(t);
                let _ap = ap_interpolator.sample(t).clamp(0.0, 1.0);

                new_sp[i][j] = _sp;
                new_ap[i][j] = _ap;
            }
        }

        *sp = new_sp;
        *ap = new_ap;
        
        *f0_off = f0_off_interpolator.evaluate(&timing.positions)?.to_vec();

        Ok(t_render)
    }

    
    fn process_prerender_flags(&self, sp: &mut Vec<Vec<f64>>, ap: &mut Vec<Vec<f64>>, f0: &mut Vec<f64>, t: &Vec<f64>, timing: &TimingData) -> Result<()> {
        // Process vocal fry
        if let Some(fry_end) = self.flags.fry_end {
            let fry = fry_end as f64 / 1000.0;
            let mut fry_len = 0.075f64;
            let mut fry_offset = 0.0;
            let mut fry_pitch = F0_FLOOR;

            if let Some(_fry_len) = self.flags.fry_length {
                fry_len = (_fry_len as f64 / 1000.0).max(0.001);
            }
            if let Some(_fry_offset) = self.flags.fry_offset {
                fry_offset = _fry_offset as f64 / 1000.0;
            }
            if let Some(_fry_pitch) = self.flags.fry_pitch {
                fry_pitch = _fry_pitch.max(0) as f64;
            }

            let t_fry = t.iter().map(|x| x - t[timing.con as usize] - fry_offset).collect::<Array1<f64>>();
            let amt = smoothstep(-fry - fry_len / 2.0, -fry + fry_len / 2.0, &t_fry) * smoothstep(fry_len / 2.0, -fry_len / 2.0, &t_fry);

            *f0 = f0.iter().zip(amt.iter()).map(|(f, a)| f + fry_pitch * a).collect();
        }

        // Process Gender flag
        if let Some(gender) = self.flags.gender {
            let gender = (gender as f64 / 120.0).pow(2.0);
            let freq_x = linspace(0.0, 1.0, (get_fft_size() as f32 / 2.0).floor() as usize + 1);
            let freq_x2 = linspace(0.0, gender, (get_fft_size() as f32 / 2.0).floor() as usize + 1).iter().map(|x| x.max(0.0).min(1.0)).collect::<Vec<f64>>();

            let mut new_sp = vec![vec![0.0; sp[0].len()]; t.len()];

            for i in 0..sp.len() {
                let sp_points = vec_to_points(&freq_x, &sp[i]);
                let sp_interpolator = Spline::from_vec(sp_points);

                for (j, x) in freq_x2.iter().enumerate() {
                    new_sp[i][j] = sp_interpolator.sample(*x);
                }
            }

            *sp = new_sp;
        }

        Ok(())
    }
}