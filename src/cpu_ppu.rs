#[derive(Copy, Clone, Debug)]
pub struct Nmi {
    pub cycle : u16,
}

pub struct PpuTime {
    pub scanline: i16,
    pub cycle: u16,
    pub frame: u128,
}

pub trait PpuState {
    fn was_nmi_triggered(&self) -> Option<Nmi>;
    fn get_time(&self) -> PpuTime;
}
