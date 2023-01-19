use crate::metadata::{MetadataHeader, MetadataHeap, StreamHeader, StreamHeaderIterator};
use crate::portable_executable::{DataDirectory, PeHeader, PeOptionalHeader, SectionHeader};
use crate::{ParsingError, ZeroCopyReader};
use std::collections::HashMap;
use std::any::{Any, TypeId};
use crate::ParsingError::*;
use std::ops::Deref;

pub struct Assembly<'l> {
    bytes: &'l [u8],
    pe_header: &'l PeHeader,
    pe_optional_header: &'l PeOptionalHeader,
    cli_header: &'l CliHeader,
    metadata_header: MetadataHeader<'l>,
    heaps: HashMap<TypeId, Box<dyn MetadataHeap<'l> + 'l>>,
}

impl<'l> TryFrom<&'l [u8]> for Assembly<'l> {
    type Error = ParsingError;

    fn try_from(bytes: &'l [u8]) -> Result<Self, Self::Error> {
        let mut reader = ZeroCopyReader::new(bytes);

        reader.seek(0x3c)?;
        let pe_start = (reader.read::<u32>()? + 4) as usize;
        reader.seek(pe_start)?;

        let pe_header = reader.read::<PeHeader>()?;
        let pe_optional_header = reader.read::<PeOptionalHeader>()?;

        let sections = reader.read_slice::<SectionHeader>(pe_header.number_of_sections as usize)?;

        let cli_header_start = resolve_rva(pe_optional_header.data_directories[14].rva, &sections)?;
        reader.seek(cli_header_start)?;
        let cli_header = reader.read::<CliHeader>()?;

        let metadata_start = resolve_rva(cli_header.metadata.rva, &sections)?;
        let metadata_header = MetadataHeader::new(bytes, metadata_start)?;
        let heaps = metadata_header.get_heaps(bytes);

        Ok(Assembly {
            bytes,
            pe_header,
            pe_optional_header,
            cli_header,
            heaps,
            metadata_header,
        })
    }
}

impl Assembly<'_> {
    pub fn get_metadata_heap<'l, T: MetadataHeap<'l> + 'static>(&self) -> Option<&'l T> {
        let heap = self.heaps.get(&TypeId::of::<T>())?.deref();
        let ptr = heap as *const _ as *const T;
        unsafe { Some(&*ptr) }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct CliHeader {
    pub size: u32,
    pub major_runtime_version: u16,
    pub minor_runtime_version: u16,
    pub metadata: DataDirectory,
    pub flags: u32,
    pub entry_point_token: u32,
    pub resources: DataDirectory,
    pub strong_name_signature_rva: u64,
    pub code_manager_table: u64,
    pub v_table_fixups_rva: u64,
    pub export_address_table_jumps: u64,
    pub managed_native_header: u64,
}

pub mod runtime_flags {
    pub const COMIMAGE_FLAGS_ILONLY: u32 = 0x00000001;
    pub const COMIMAGE_FLAGS_32BITREQUIRED: u32 = 0x00000002;
    pub const COMIMAGE_FLAGS_STRONGNAMESIGNED: u32 = 0x00000008;
    pub const COMIMAGE_FLAGS_NATIVE_ENTRYPOINT: u32 = 0x00000010;
    pub const COMIMAGE_FLAGS_TRACKDEBUGDATA: u32 = 0x00010000;
}

fn resolve_rva(rva: u32, sections: &[SectionHeader]) -> Result<usize, ParsingError> {
    let section = sections
        .iter()
        .find(|s| rva >= s.virtual_address && rva < (s.virtual_address + s.size_of_raw_data))
        .ok_or(MissingSection)?;

    Ok((rva - section.virtual_address + section.pointer_to_raw_data) as usize)
}
