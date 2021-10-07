use crate::common::Mirroring;

mod mapper0;
mod mapper1;
mod mapper2;
mod mapper227;
mod mapper4;
mod mapper66;
mod mapper7;
mod mapper71;
mod mapper_null;
mod mmc3_6;

mod mapper_internal;

pub use self::mapper0::Mapper0;
pub use self::mapper1::Mapper1;
pub use self::mapper2::Mapper2;
pub use self::mapper227::Mapper227;
pub use self::mapper4::Mapper4;
pub use self::mapper66::Mapper66;
pub use self::mapper7::Mapper7;
pub use self::mapper71::Mapper71;
pub use self::mapper_null::MapperNull;

pub trait Mapper: erased_serde::Serialize {
    fn get_mapper_id(&self) -> u8;

    fn get_chr_byte(&mut self, _: u16) -> u8;
    fn store_chr_byte(&mut self, _: u16, _: u8);

    fn get_prg_byte(&mut self, _: u16) -> u8;
    fn store_prg_byte(&mut self, _: u16, _: u8);

    fn get_mirroring(&self) -> Mirroring;
    fn power_cycle(&mut self);

    fn is_irq_pending(&mut self) -> bool {
        false
    }

    fn ppu_a12_rising_edge_triggered(&mut self) {}
}

erased_serde::serialize_trait_object!(Mapper);

// pub trait BoxToRcRefCell {
//     fn wrap_in_refcell(self: Box<Self>) -> Rc<RefCell<dyn Mapper>>;
// }
// impl<T: Mapper + 'static> BoxToRcRefCell for T {
//     fn wrap_in_refcell(self: Box<Self>) -> Rc<RefCell<dyn Mapper>> {
//         Rc::new(RefCell::new(*self))
//     }
// }
