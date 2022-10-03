use std::{
    default,
    fmt::{self, Debug, DebugStruct, Formatter},
    fs::File,
    io::{Error, Read, Write},
    mem, slice,
};

#[repr(C)]
pub struct ROM {
    skip: [u8; 0x100],
    header: ROMHeader,
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
            skip: [0; 0x100],
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
                glob_chk: [0; 0x2], 
            }
        }
    }
}

impl ROM {
    pub fn load(mut reader: impl Read) -> Result<Self, Error> {
        let mut rom = ROM::default();
        unsafe {
            // Get a slice which treats the `speaker` variable as a byte array
            let buffer: &mut [u8] = std::slice::from_raw_parts_mut(
                &mut rom as *mut ROM as *mut u8,
                mem::size_of::<ROM>(),
            );

            // Read exactly that many bytes from the reader
            reader.read_exact(buffer)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;

    #[test]
    fn deserialize_rom() {
        let path =
            Path::new("./test/data/Legend of Zelda, The - Link's Awakening (U) (V1.2) [!].gb");
        let f = File::open(&path).expect("Unable to open file");
        let mut reader = BufReader::new(f);

        let got = ROM::load(reader).unwrap();
        assert_eq!(hex::encode(got.header.nin_logo), 
            "ceed6666cc0d000b03730083000c000d0008111f8889000edccc6ee6ddddd999bbbb67636e0eecccdddc999fbbb9333e");

        let hdr_calc = got.hdr_chk();
        assert_eq!(hdr_calc, got.header.hdr_chk);
    }
}