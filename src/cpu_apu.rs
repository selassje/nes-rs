pub trait ApuState {
    fn is_irq_pending(&self) -> bool;
}
