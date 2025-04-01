//! Helper functions for parsing ELF files

use std::fs;
use std::path::PathBuf;

use object::elf;
use object::read::elf::FileHeader;

use crate::error::ElfError;
use crate::error::SimulatorResult;

pub type ELFReaderType = elf::FileHeader32<object::Endianness>;
pub type Segment = elf::ProgramHeader32<object::Endianness>;

/// Returns the program entry address
pub fn get_elf_entry(elf_reader: &ELFReaderType) -> SimulatorResult<u32> {
    let endian = get_elf_endian(elf_reader)?;
    Ok(elf_reader.e_entry(endian))
}

/// Returns the pair (ELF reader, binary data)
pub fn parse_elf_file(
    file_path: &str,
) -> SimulatorResult<(ELFReaderType, Vec<u8>)> {
    let path = PathBuf::from(file_path);

    let data = fs::read(&path)
        .map_err(|e| ElfError::FileReadError(path.clone(), e))?;

    let elf = elf::FileHeader32::<object::Endianness>::parse(&*data)
        .map_err(|e| ElfError::ParseError(path, e.to_string()))?;

    Ok((*elf, data))
}

/// Returns the endianness
pub fn get_elf_endian(
    elf_reader: &ELFReaderType,
) -> SimulatorResult<object::Endianness> {
    elf_reader
        .endian()
        .map_err(|e| ElfError::InvalidFormat(e.to_string()).into())
}

/// Returns the machine type
pub fn get_elf_machine(elf_reader: &ELFReaderType) -> SimulatorResult<u16> {
    let endian = get_elf_endian(elf_reader)?;
    Ok(elf_reader.e_machine(endian))
}

/// Return it as a vector for good
pub fn get_elf_segments(
    elf_reader: &ELFReaderType,
    elf_data: &[u8],
) -> SimulatorResult<Vec<Segment>> {
    let endian = get_elf_endian(elf_reader)?;

    elf_reader
        .program_headers(endian, elf_data)
        .map_err(|e| ElfError::InvalidFormat(e.to_string()).into())
        .map(|headers| headers.to_vec())
}
