pub struct TimingData {
    pub positions: Vec<f64>,
    pub start: f32,
    pub end: f32,
    pub con: f32
}

impl TimingData {
    pub fn calculate(f0_len: usize, offset: f32, cutoff: f32, consonant: f32) -> TimingData {
        // Calculate timing
        log::info!("Calculating timing");

        let mut t_area = vec![];
        for i in 0..f0_len {
            t_area.push(i as f64 * 0.005);
        }
        
        let start: f32 = offset / 1000.0;
        let end = cutoff / 1000.0;
        let end = if cutoff < 0.0 {
            start - end
        } else {
            t_area.last().unwrap().clone() as f32 - end
        };

        let con = start + consonant / 1000.0;

        TimingData {
            positions: t_area,
            start,
            end,
            con
        }
    }
}