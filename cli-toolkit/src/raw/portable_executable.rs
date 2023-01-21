use std::ffi::c_char;
use crate::raw::FromByteStream;

#[repr(C)]
#[derive(Debug, Clone, FromByteStream)]
pub struct PeHeader {
	#[check_value(|v| *v == 0x4550)]
	magic: u32,
	#[check_value(|v| *v == 0x014C)]
	pub machine: u16,
	pub number_of_sections: u16,
	pub timestamp: i32,
	#[check_value(|v| *v == 0)]
	pub pointer_to_symbol_table: u32,
	#[check_value(|v| *v == 0)]
	pub number_of_symbols: u32,
	pub optional_header_size: u16,
	#[check_value(|v| *v & 0x000F == 0x2)]
	pub characteristics: u16,
}

#[repr(C)]
#[derive(Debug, Clone, FromByteStream)]
pub struct PeOptionalHeader {
	pub standard_fields: StandardFields,
	pub nt_specific_fields: NTSpecificFields,
	pub data_directories: [DataDirectory; 16],
}

#[repr(C)]
#[derive(Debug, Clone, FromByteStream)]
pub struct StandardFields {
	#[check_value(|v| *v == 0x10B)]
	pub magic: u16,
	#[check_value(|v| *v == 6)]
	pub l_major: u8,
	#[check_value(|v| *v == 0)]
	pub l_minor: u8,
	pub code_size: u32,
	pub initialized_data_size: u32,
	pub uninitialized_data_size: u32,
	pub entry_point_rva: u32,
	pub base_of_code: u32,
	pub base_of_data: u32,
}

#[repr(C)]
#[derive(Debug, Clone, FromByteStream)]
pub struct NTSpecificFields {
	#[check_value(|v| *v % 0x10000 == 0)]
	pub image_base: u32,
	#[validate_value(|v: &u32| *v > file_alignment)]
	pub section_alignment: u32,
	#[check_value(|v| *v == 0x200)]
	pub file_alignment: u32,
	#[check_value(|v| *v == 5)]
	pub os_major: u16,
	#[check_value(|v| *v == 0)]
	pub os_minor: u16,
	#[check_value(|v| *v == 0)]
	pub user_major: u16,
	#[check_value(|v| *v == 0)]
	pub user_minor: u16,
	#[check_value(|v| *v == 5)]
	pub sub_sys_major: u16,
	#[check_value(|v| *v == 0)]
	pub sub_sys_minor: u16,
	#[check_value(|v| *v == 0)]
	reserved: u32,
	#[check_value(|v| *v % section_alignment == 0)]
	pub image_size: u32,
	#[check_value(|v| *v % file_alignment == 0)]
	pub header_size: u32,
	#[check_value(|v| *v == 0)]
	pub file_checksum: u32,
	#[check_value(|v| *v == 0x2 || *v == 0x3)]
	pub sub_system: u16,
	#[check_value(|v| *v & 0x100F == 0)]
	pub dll_flags: u16,
	#[check_value(|v| *v == 0x100000)]
	pub stack_reserve_size: u32,
	#[check_value(|v| *v == 0x1000)]
	pub stack_commit_size: u32,
	#[check_value(|v| *v == 0x100000)]
	pub heap_reserve_size: u32,
	#[check_value(|v| *v == 0x1000)]
	pub heap_commit_size: u32,
	#[check_value(|v| *v == 0)]
	pub loader_flags: u32,
	#[check_value(|v| *v == 0x10)]
	pub number_of_data_directories: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Eq, PartialEq, FromByteStream)]
pub struct DataDirectory {
	pub rva: u32,
	pub size: u32,
}

#[repr(C)]
#[derive(Debug, Clone, FromByteStream)]
pub struct DataDirectories {
	#[check_value(|v| *v == Default::default())]
	pub export_table: DataDirectory,
	pub import_table: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub resource_table: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub exception_table: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub certificate_table: DataDirectory,
	pub base_relocation_table: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub debug: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub copyright: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub global_ptr: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub tls_table: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub load_config_table: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub bound_import: DataDirectory,
	pub import_address_table: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	pub delay_import_descriptor: DataDirectory,
	pub cli_header: DataDirectory,
	#[check_value(|v| *v == Default::default())]
	reserved: DataDirectory,
}

#[repr(C)]
#[derive(Debug, Clone, FromByteStream)]
pub struct SectionHeader {
	pub name: u64,
	pub virtual_size: u32,
	pub virtual_address: u32,
	pub size_of_raw_data: u32,
	pub pointer_to_raw_data: u32,
	#[check_value(|v| *v == 0)]
	pub pointer_to_relocations: u32,
	#[check_value(|v| *v == 0)]
	pub pointer_to_line_numbers: u32,
	#[check_value(|v| *v == 0)]
	pub number_of_relocations: u16,
	#[check_value(|v| *v == 0)]
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
#[derive(Debug, Clone, FromByteStream)]
pub struct ImportTable {
	pub import_lookup_table_rva: u32,
	#[check_value(|v| *v == 0)]
	pub date_time_stamp: i32,
	#[check_value(|v| *v == 0)]
	pub forwarder_chain: u32,
	pub name: u32,
	pub import_address_table_rva: u32,
	end_padding: [u8; 20],
}

pub mod pe_header_characteristics {
	pub const IMAGE_FILE_RELOCS_STRIPPED: u16 = 0x01;
	pub const IMAGE_FILE_EXECUTABLE_IMAGE: u16 = 0x02;
	pub const IMAGE_FILE_32BIT_MACHINE: u16 = 0x0100;
	pub const IMAGE_FILE_DLL: u16 = 0x2000;
}

pub mod section_header_characteristics {
	pub const IMAGE_SCN_CNT_CODE: u32 = 0x20;
	pub const IMAGE_SCN_CNT_INITIALIZED_DATA: u32 = 0x40;
	pub const IMAGE_SCN_CNT_UNINITIALIZED_DATA: u32 = 0x80;
	pub const IMAGE_SCN_MEM_EXECUTE: u32 = 0x20000000;
	pub const IMAGE_SCN_MEM_READ: u32 = 0x40000000;
	pub const IMAGE_SCN_MEM_WRITE: u32 = 0x80000000;
}

#[repr(C)]
#[derive(Debug, Clone, FromByteStream)]
pub struct CliHeader {
	pub size: u32,
	pub major_runtime_version: u16,
	pub minor_runtime_version: u16,
	pub metadata: DataDirectory,
	#[check_value(|v| {
		(*v & runtime_flags::IL_ONLY == 1) &&
		(*v & runtime_flags::NATIVE_ENTRYPOINT == 0) &&
		(*v & runtime_flags::TRACK_DEBUG_DATA == 0)
	})]
	pub flags: u32,
	pub entry_point_token: u32,
	pub resources: DataDirectory,
	pub strong_name_signature_rva: u64,
	#[check_value(|v| *v == 0)]
	pub code_manager_table: u64,
	pub v_table_fixups_rva: u64,
	#[check_value(|v| *v == 0)]
	pub export_address_table_jumps: u64,
	#[check_value(|v| *v == 0)]
	pub managed_native_header: u64,
}

pub mod runtime_flags {
	pub const IL_ONLY: u32 = 0x01;
	pub const REQUIRE_32BIT: u32 = 0x02;
	pub const STRONG_NAME_SIGNED: u32 = 0x08;
	pub const NATIVE_ENTRYPOINT: u32 = 0x10;
	pub const TRACK_DEBUG_DATA: u32 = 0x10000;
}
