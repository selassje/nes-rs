use serde::{Deserialize, Deserializer, Serialize};

trait BoxedArrayDeserialize<'de>: Sized {
    fn deserialize<D, const N: usize>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>;
}

impl<'de, T, const M: usize> BoxedArrayDeserialize<'de> for Box<[T; M]>
where
    T: Default + Copy + Deserialize<'de>,
{
    fn deserialize<D, const N: usize>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use std::marker::PhantomData;
        struct ArrayVisitor<T, const N: usize> {
            element: PhantomData<T>,
        }

        impl<'de, T, const N: usize> serde::de::Visitor<'de> for ArrayVisitor<T, N>
        where
            T: Default + Copy + serde::Deserialize<'de>,
        {
            type Value = Box<[T; N]>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                macro_rules! write_len {
                    ($l:literal) => {
                        write!(formatter, concat!("an array of length ", $l))
                    };
                    ($l:tt) => {
                        write!(formatter, "an array of length {}", $l)
                    };
                }

                write_len!(N)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut arr = Box::new([T::default(); N]);
                for i in 0..N {
                    arr[i] = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(arr)
            }
        }

        let visitor = ArrayVisitor {
            element: PhantomData,
        };
        #[allow(unused_parens)]
        deserializer.deserialize_tuple(N, visitor)
    }
}

const PRG_RAM_DATA_SIZE: usize = 0x20000;
const PRG_ROM_DATA_SIZE: usize = 0x80000;
const CHR_ROM_DATA_SIZE: usize = 0x40000;
const CHR_RAM_DATA_SIZE: usize = 0x2000;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub(super) enum BankSize {
    _1KB = 0x0400,
    _2KB = 0x0800,
    _4KB = 0x1000,
    _8KB = 0x2000,
    _16KB = 0x4000,
    _32KB = 0x8000,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(super) struct BankSelect {
    pub size: BankSize,
    pub bank: usize,
}

impl Default for BankSelect {
    fn default() -> Self {
        Self {
            size: BankSize::_1KB,
            bank: 0,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub(super) struct MapperInternal {
    #[serde(
        serialize_with = "serde_arrays::serialize",
        deserialize_with = "BoxedArrayDeserialize::deserialize::<_,PRG_RAM_DATA_SIZE>"
    )]
    prg_ram: Box<[u8; PRG_RAM_DATA_SIZE]>,
    #[serde(
        serialize_with = "serde_arrays::serialize",
        deserialize_with = "BoxedArrayDeserialize::deserialize::<_,PRG_ROM_DATA_SIZE>"
    )]
    prg_rom: Box<[u8; PRG_ROM_DATA_SIZE]>,
    prg_rom_size: usize,
    #[serde(
        serialize_with = "serde_arrays::serialize",
        deserialize_with = "BoxedArrayDeserialize::deserialize::<_,CHR_ROM_DATA_SIZE>"
    )]
    chr_rom: Box<[u8; CHR_ROM_DATA_SIZE]>,
    chr_rom_size: usize,
    #[serde(
        serialize_with = "serde_arrays::serialize",
        deserialize_with = "BoxedArrayDeserialize::deserialize::<_,CHR_RAM_DATA_SIZE>"
    )]
    chr_ram: Box<[u8; CHR_RAM_DATA_SIZE]>,
}

impl MapperInternal {
    pub fn new(_prg_rom: Vec<u8>, _chr_rom: Vec<u8>) -> Self {
        let mut prg_rom = Box::new([0; PRG_ROM_DATA_SIZE]);
        let mut chr_rom = Box::new([0; CHR_ROM_DATA_SIZE]);
        chr_rom[.._chr_rom.len()].copy_from_slice(_chr_rom.as_slice());
        prg_rom[.._prg_rom.len()].copy_from_slice(_prg_rom.as_slice());
        Self {
            prg_ram: Box::new([0; PRG_RAM_DATA_SIZE]),
            prg_rom,
            prg_rom_size: _prg_rom.len(),
            chr_ram: Box::new([0; CHR_RAM_DATA_SIZE]),
            chr_rom,
            chr_rom_size: _chr_rom.len(),
        }
    }

    fn get_address_index(&self, address: u16, bank: usize, bank_size: BankSize) -> usize {
        bank_size as usize * bank + (address as usize % bank_size as usize)
    }

    pub fn get_prg_rom_byte(&mut self, address: u16, bank: usize, prg_bank_size: BankSize) -> u8 {
        let index = self.get_address_index(address, bank, prg_bank_size);
        self.prg_rom[index]
    }

    pub fn get_prg_ram_byte(&mut self, address: u16, bank: usize, bank_size: BankSize) -> u8 {
        self.prg_ram[self.get_address_index(address, bank, bank_size)]
    }

    pub fn store_prg_ram_byte(&mut self, address: u16, bank: usize, bank_size: BankSize, byte: u8) {
        self.prg_ram[self.get_address_index(address, bank, bank_size)] = byte
    }

    pub fn get_chr_byte(&mut self, address: u16, bank: usize, chr_bank_size: BankSize) -> u8 {
        if self.chr_rom_size == 0 {
            self.chr_ram[address as usize]
        } else {
            let index = self.get_address_index(address, bank, chr_bank_size);
            self.chr_rom[index]
        }
    }

    pub fn store_chr_byte(&mut self, address: u16, _: usize, _: BankSize, byte: u8) {
        self.chr_ram[address as usize] = byte;
    }

    pub fn get_prg_rom_bank_count(&self, prg_bank_size: BankSize) -> usize {
        self.prg_rom_size / prg_bank_size as usize
    }

    pub fn reset(&mut self) {
        self.chr_ram.iter_mut().for_each(|m| *m = 0);
        self.prg_ram.iter_mut().for_each(|m| *m = 0);
    }
}
