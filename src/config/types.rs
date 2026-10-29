pub type Rgb = [f64; 3];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Metric {
    Ram,
    Io,
    Cpu,
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub ok: Rgb,
    pub warn: Rgb,
    pub hot: Rgb,
}

pub struct Colors {
    pub light: bool,
    pub ink: Option<Rgb>,
    pub empty: Rgb,
    pub ctx_from: Rgb,
    pub ctx_to: Rgb,
}
