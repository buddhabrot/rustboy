use std::{
    default,
    fmt::{self, DebugStruct, Formatter},
    fs::File,
    io::{Error, Read, Write, BufReader, Seek},
    mem, slice,
};

#[repr(C)]
pub struct ROM {
    bios: Vec<u8>,
    header: ROMHeader,
    dat: Vec<u8> // just allocate 8 MiB
}

#[repr(C)]
pub struct ROMHeader {
    entry_point: [u8; 0x4],
    nin_logo: [u8; 0x30],
    title: [u8; 0x10],
    new_lic_code: [u8; 0x2],
    sgb_flag: u8,
    ctdg_type: u8,
    rom_size: u8,
    ram_size: u8,
    dst_code: u8,
    old_lic_code: u8,
    rom_version: u8,
    hdr_chk: u8,
    glob_chk: [u8; 0x2]
}

impl Default for ROM {
    fn default() -> Self {
        ROM {
            bios: vec![],
            header: ROMHeader { 
                entry_point: [0; 0x4], 
                nin_logo: [0; 0x30],
                title: [0; 0x10],
                new_lic_code: [0; 0x2],
                sgb_flag: 0,
                ctdg_type: 0,
                rom_size: 0,
                ram_size: 0,
                dst_code: 0,
                old_lic_code: 0,
                rom_version: 0,
                hdr_chk: 0,
                glob_chk: [0; 0x2]
            },
            dat: vec![]
        }
    }
}

impl ROM {
    pub fn load(mut reader: BufReader<File>, mut bios_reader: BufReader<File>) -> Result<Self, Error> {
        let mut rom = ROM::default();

        unsafe {
            // TODO: protect against rom or bios too large/small

            // Read bios into vec
            bios_reader.read_to_end(&mut rom.bios).expect("Unable to read bios");

            let buffer: &mut [u8] = std::slice::from_raw_parts_mut(
                &mut rom.header as *mut ROMHeader as *mut u8,
                mem::size_of::<ROMHeader>(),
            );

            // Read header into static struct
            reader.seek(std::io::SeekFrom::Start(0x100)).expect("Unable to read header");

            let header_length = mem::size_of::<ROMHeader>() as u64;
            reader.by_ref().take(header_length).read_exact(buffer)?;

            // Read rest into vec
            reader.seek(std::io::SeekFrom::Start(0x100 + header_length)).expect("Unable to read rom");
            reader.read_to_end(&mut rom.dat)?;

            Ok(rom)
        }
    }

    pub fn hdr_chk(&self) -> u8 {
        //x=0:FOR i=0134h TO 014Ch:x=x-MEM[i]-1:NEXT
        let mut chk: u8 = 0;
        let data: *const u8 = self as *const ROM as *const u8;
        unsafe {
            (0x134..=0x14c).for_each(|n| {
                let new_p = data.offset(n);
                chk = chk.wrapping_sub(*new_p).wrapping_sub(1);
             });
        }
        return (chk & 0xFF) as u8;
    }

    pub fn rom_size_bytes(&self) -> u32 {
        match self.header.rom_size {
            0x0..=0x8 => return 32 * (1 << self.header.rom_size) * 0x400,
            _ => return 32 * 0x400,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    #[test]
    fn deserialize_rom() {
        let path = Path::new("./test/data/Legend of Zelda, The - Link's Awakening (U) (V1.2) [!].gb");
        let file = File::open(&path).expect("Unable to open test file");
        let mut reader = BufReader::new(file);

        let bios_path =
            Path::new("./bios/gb_bios.bin");
        let bios_file = File::open(&bios_path).expect("Unable to open bios");
        let mut bios_reader = BufReader::new(bios_file);

        
        let got = ROM::load(reader, bios_reader).expect("Unable to load rom");
        /*assert_eq!(hex::encode(got.header.nin_logo), 
            "ceed6666cc0d000b03730083000c000d0008111f8889000edccc6ee6ddddd999bbbb67636e0eecccdddc999fbbb9333e");

        let hdr_calc = got.hdr_chk();
        assert_eq!(hdr_calc, got.header.hdr_chk);*/
    }
}