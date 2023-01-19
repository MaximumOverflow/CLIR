use std::ffi::c_char;

#[repr(C)]
#[derive(Debug)]
pub struct PeHeader {
    pub machine: u16,
    pub number_of_sections: u16,
    pub timestamp: i32,
    pub pointer_to_symbol_table: u32,
    pub number_of_symbols: u32,
    pub optional_header_size: u16,
    pub characteristics: u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct PeOptionalHeader {
    pub standard_fields: StandardFields,
    pub nt_specific_fields: NTSpecificFields,
    pub data_directories: [DataDirectory; 16],
}

#[repr(C)]
#[derive(Debug)]
pub struct StandardFields {
    pub magic: u16,
    pub l_major: u8,
    pub l_minor: u8,
    pub code_size: u32,
    pub initialized_data_size: u32,
    pub uninitialized_data_size: u32,
    pub entry_point_rva: u32,
    pub base_of_code: u32,
    pub base_of_data: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct NTSpecificFields {
    pub image_base: u32,
    pub section_alignment: u32,
    pub file_alignment: u32,
    pub os_major: u16,
    pub os_minor: u16,
    pub user_major: u16,
    pub user_minor: u16,
    pub sub_sys_major: u16,
    pub sub_sys_minor: u16,
    reserved: u32,
    pub image_size: u32,
    pub header_size: u32,
    pub file_checksum: u32,
    pub sub_system: u16,
    pub dll_flags: u16,
    pub stack_reserve_size: u32,
    pub stack_commit_size: u32,
    pub heap_reserve_size: u32,
    pub heap_commit_size: u32,
    pub loader_flags: u32,
    pub number_of_data_directories: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct DataDirectory {
    pub rva: u32,
    pub size: u32,
}

#[repr(C)]
#[derive(Debug)]
pub struct DataDirectories {
    pub export_table: DataDirectory,
    pub import_table: DataDirectory,
    pub resource_table: DataDirectory,
    pub exception_table: DataDirectory,
    pub certificate_table: DataDirectory,
    pub base_relocation_table: DataDirectory,
    pub debug: DataDirectory,
    pub copyright: DataDirectory,
    pub global_ptr: DataDirectory,
    pub tls_table: DataDirectory,
    pub load_config_table: DataDirectory,
    pub bound_import: DataDirectory,
    pub import_address_table: DataDirectory,
    pub delay_import_descriptor: DataDirectory,
    pub cli_header: DataDirectory,
    reserved: DataDirectory,
}

#[repr(C)]
#[derive(Debug)]
pub struct SectionHeader {
    pub name: u64,
    pub virtual_size: u32,
    pub virtual_address: u32,
    pub size_of_raw_data: u32,
    pub pointer_to_raw_data: u32,
    pub pointer_to_relocations: u32,
    pub pointer_to_line_numbers: u32,
    pub number_of_relocations: u16,
    pub number_of_line_numbers: u16,
    pub characteristics: u32,
}

impl SectionHeader {
    pub fn name(&self) -> &str {
        unsafe {
            let ptr = &self.name as *const u64 as *const c_char;
            std::ffi::CStr::from_ptr(ptr).to_str().unwrap()
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ImportTable {
    pub import_lookup_table_rva: u32,
    pub date_time_stamp: i32,
    pub forwarder_chain: u32,
    pub name: u32,
    pub import_address_table_rva: u32,
    end_padding: [u8; 20],
}

pub mod pe_header_characteristics {
    pub const IMAGE_FILE_RELOCS_STRIPPED: u16 = 0x0001;
    pub const IMAGE_FILE_EXECUTABLE_IMAGE: u16 = 0x0002;
    pub const IMAGE_FILE_32BIT_MACHINE: u16 = 0x0100;
    pub const IMAGE_FILE_DLL: u16 = 0x2000;
}

pub mod section_header_characteristics {
    pub const IMAGE_SCN_CNT_CODE: u32 = 0x00000020;
    pub const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x00000040;
    pub const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x00000080;
    pub const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
    pub const IMAGE_SCN_MEM_READ: u32 = 0x40000000;
    pub const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;
}
