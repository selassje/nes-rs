#[derive(Copy, Clone, Debug)]
pub enum Nmi {
    VblankStart,
    ImmediateOccurence,
}

pub trait PpuNmiState {
    fn was_nmi_triggered(&self) -> Option<Nmi>;
}
