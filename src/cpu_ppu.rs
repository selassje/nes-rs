pub struct PpuTime {
    pub scanline: i16,
    pub cycle: u16,
    pub frame: u128,
}

pub trait PpuState {
    fn is_nmi_pending(&mut self) -> bool;
    fn clear_nmi_pending(&mut self);
    fn get_time(&self) -> PpuTime;
}
