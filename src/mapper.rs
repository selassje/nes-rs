pub trait Mapper {
    fn get_rom_start(&self) -> u16;

    fn get_pgr_rom(&self) -> &[u8];
}

pub struct Mapper0 {
    prg_rom: Vec::<u8>,
    prg_ram: Vec::<u8>,
}

impl Mapper0 {

    pub fn new(prg_rom: Vec::<u8>, prg_ram_size: u32) -> Mapper0 {
            let mut final_pgr_rom = prg_rom.clone();
            if final_pgr_rom.len() <= 16384 {
                final_pgr_rom.extend_from_slice(prg_rom.as_slice())
            }
         
            Mapper0{
                prg_rom : final_pgr_rom,
                prg_ram : vec![0; prg_ram_size as usize],
            }
    }
}

impl Mapper for Mapper0 {

    fn get_rom_start(&self) -> u16 {
        0x8000
    }

    fn get_pgr_rom(&self) -> &[u8] {
        self.prg_rom.as_slice()
    }
}


