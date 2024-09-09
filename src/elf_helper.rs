//! Helper functions for parsing ELF files

use object::elf;
use object::read::elf::FileHeader;
use std::error::Error;
use std::fs;

pub type ELFReaderType = elf::FileHeader32<object::Endianness>;
pub type Segment = elf::ProgramHeader32<object::Endianness>;

/// Returns the program entry address
pub fn get_elf_entry(elf_reader: &ELFReaderType) -> Result<u32, Box<dyn Error>> {
    let endian = get_elf_endian(elf_reader)?;
    return Ok(elf_reader.e_entry(endian));
}

/// Returns the pair (ELF reader, binary data)
pub fn parse_elf_file(
    file_path: &str,
) -> Result<(ELFReaderType, Vec<u8>), Box<dyn Error>> {
    let data = fs::read(file_path)?;
    let elf = elf::FileHeader32::<object::Endianness>::parse(&*data)?;
    return Ok((*elf, data));
}

/// Returns the endianness
pub fn get_elf_endian(
    elf_reader: &ELFReaderType,
) -> Result<object::Endianness, Box<dyn Error>> {
    return Ok(elf_reader.endian()?);
}

/// Returns the machine type
pub fn get_elf_machine(elf_reader: &ELFReaderType) -> Result<u16, Box<dyn Error>> {
    return Ok(elf_reader.e_machine(get_elf_endian(elf_reader)?));
}

/// Return it as a vector for good
pub fn get_elf_segments(
    elf_reader: &ELFReaderType,
    elf_data: &[u8],
) -> Result<Vec<Segment>, Box<dyn Error>> {
    let endian = get_elf_endian(elf_reader)?;
    return Ok(elf_reader.program_headers(endian, elf_data)?.to_vec());
}
