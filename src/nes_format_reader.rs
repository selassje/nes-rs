#[derive(PartialEq, Debug)]
enum NesFormat {
    Nes2_0,
    INes,    
}


enum HeaderFlag6 {
    MirroringVertical   = 0b00000001,
    PgrRAMPresent       = 0b00000010,
    TrainerPresent      = 0b00000100,
    IgnoreMirroring     = 0b00001000,
}



struct NesHeader {
    pgr_rom_units      : u8,
    chr_rom_units      : u8,
    flag_6             : HeaderFlag6,
    lo_n_mapper_number : u8




}


pub struct NesFile {
  
}

impl NesFile {

    pub fn new(in_bytes : &Vec<u8>) -> NesFile {
        let format = Self::get_format(in_bytes);
        assert!(format == NesFormat::INes);
        NesFile{}
    }

    fn get_format(header : &Vec<u8>) -> NesFormat {
        let mut is_ines_format = false;
        let mut is_nes2_format = false;
        if header[0] == 'N' as u8 && header[1] == 'E' as u8 && header[2]== 'S' as u8 && header[3] == 0x1A {
            is_ines_format = true;
        }

        if is_ines_format && (header[7] & 0x0C == 0x1A) {
            is_nes2_format = true;
        }

        if is_nes2_format {NesFormat::Nes2_0}
        else if is_ines_format {NesFormat::INes}
        else {panic!("Unknown format file")}
    }
}