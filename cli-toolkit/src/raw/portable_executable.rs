use std::ffi::c_char;
use crate::raw::{ByteStream, Error, FromByteStream};

#[repr(C)]
#[derive(Debug, Clone, FromByteStream)]
pub struct DosHeader {
	#[check_value(|v: &[u8; 128]| match v {
		[
			0x4d, 0x5a, 0x90, 0x00, 0x03, 0x00, 0x00, 0x00,
			0x04, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
			0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
			0x00, 0x00, 0x00, 0x00, _   , _   , _   , _   ,
			0x0e, 0x1f, 0xba, 0x0e, 0x00, 0xb4, 0x09, 0xcd,
			0x21, 0xb8, 0x01, 0x4c, 0xcd, 0x21, 0x54, 0x68,
			0x69, 0x73, 0x20, 0x70, 0x72, 0x6f, 0x67, 0x72,
			0x61, 0x6d, 0x20, 0x63, 0x61, 0x6e, 0x6e, 0x6f,
			0x74, 0x20, 0x62, 0x65, 0x20, 0x72, 0x75, 0x6e,
			0x20, 0x69, 0x6e, 0x20, 0x44, 0x4f, 0x53, 0x20,
			0x6d, 0x6f, 0x64, 0x65, 0x2e, 0x0d, 0x0d, 0x0a,
			0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		] => true,

		_ => false,
	})]
	bytes: [u8; 128],
}

impl DosHeader {
	pub fn lfanew(&self) -> u32 {
		u32::from_le_bytes([self.bytes[0x3C], self.bytes[0x3D], self.bytes[0x3E], self.bytes[0x3F]])
	}
}

#[repr(C)]
#[derive(Debug, Clone, FromByteStream)]
pub struct PeHeader {
	#[check_value(|v| *v == 0x4550)]
	magic: u32,
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
#[derive(Debug, Clone)]
pub struct PeOptionalHeader {
	pub standard_fields: StandardFields,
	pub nt_specific_fields: NTSpecificFields,
	pub data_directories: [DataDirectory; 16],
}

impl FromByteStream<'_> for PeOptionalHeader {
	fn from_byte_stream(reader: &mut ByteStream) -> Result<Self, Error> {
		let magic = reader.read_checked(
			|v| *v == 0x10B || *v == 0x20B,
			Some("Invalid value for PeOptionalHeader::magic"),
		)?;

		let pe64 = magic == 0x20B;

		Ok(Self {
			standard_fields: StandardFields {
				magic,
				l_major: reader.read()?,
				l_minor: reader.read()?,
				code_size: reader.read()?,
				initialized_data_size: reader.read()?,
				uninitialized_data_size: reader.read()?,
				entry_point_rva: reader.read()?,
				base_of_code: reader.read()?,
				base_of_data: if pe64 { 0 } else { reader.read()? },
			},
			nt_specific_fields: {
				let image_base = if pe64 {
					reader.read::<u64>()?
				} else {
					reader.read::<u32>()? as u64
				};

				let section_alignment = reader.read()?;
				let file_alignment =
					reader.read_checked(|v| *v == 0x200 || *v == 0x1000, Some("Invalid value for NTSpecificFields::file_alignment"))?;

				if section_alignment < file_alignment {
					return Err(Error::InvalidData(Some("Invalid value for NTSpecificFields::section_alignment")));
				}

				NTSpecificFields {
					image_base,
					section_alignment,
					file_alignment,
					os_major: reader.read()?,
					os_minor: reader.read()?,
					user_major: reader.read()?,
					user_minor: reader.read()?,
					sub_sys_major: reader.read()?,
					sub_sys_minor: reader.read()?,
					reserved: reader.read()?,
					image_size: reader.read_checked(
						|v| *v % section_alignment == 0,
						Some("Invalid value for NTSpecificFields::image_size"),
					)?,
					header_size: reader.read_checked(
						|v| *v % file_alignment == 0,
						Some("Invalid value for NTSpecificFields::header_size"),
					)?,
					file_checksum: reader.read()?,
					sub_system: reader.read_checked(
						|v| *v == 0x2 || *v == 0x3,
						Some("Invalid value for NTSpecificFields::sub_system"),
					)?,
					dll_flags: reader
						.read_checked(|v| *v & 0x100F == 0, Some("Invalid value for NTSpecificFields::dll_flags"))?,
					stack_reserve_size: if pe64 {
						reader.read_checked::<u64>(
							|v| *v == 0x400000,
							Some("Invalid value for NTSpecificFields::stack_reserve_size"),
						)?
					} else {
						reader.read_checked::<u32>(
							|v| *v == 0x100000,
							Some("Invalid value for NTSpecificFields::stack_reserve_size"),
						)? as u64
					},
					stack_commit_size: if pe64 {
						reader.read_checked::<u64>(
							|v| *v == 0x4000,
							Some("Invalid value for NTSpecificFields::stack_commit_size"),
						)?
					} else {
						reader.read_checked::<u32>(
							|v| *v == 0x1000,
							Some("Invalid value for NTSpecificFields::stack_commit_size"),
						)? as u64
					},
					heap_reserve_size: if pe64 {
						reader.read_checked::<u64>(
							|v| *v == 0x100000,
							Some("Invalid value for NTSpecificFields::heap_reserve_size"),
						)?
					} else {
						reader.read_checked::<u32>(
							|v| *v == 0x100000,
							Some("Invalid value for NTSpecificFields::heap_reserve_size"),
						)? as u64
					},
					heap_commit_size: if pe64 {
						reader.read_checked::<u64>(
							|v| *v == 0x2000,
							Some("Invalid value for NTSpecificFields::heap_commit_size"),
						)?
					} else {
						reader.read_checked::<u32>(
							|v| *v == 0x1000,
							Some("Invalid value for NTSpecificFields::heap_commit_size"),
						)? as u64
					},
					loader_flags: reader
						.read_checked(|v| *v == 0, Some("Invalid value for NTSpecificFields::loader_flags"))?,
					number_of_data_directories: reader.read_checked(
						|v| *v == 0x10,
						Some("Invalid value for NTSpecificFields::number_of_data_directories"),
					)?,
				}
			},
			data_directories: [
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
				DataDirectory::from_byte_stream(reader)?,
			],
		})
	}
}

#[repr(C)]
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct NTSpecificFields {
	pub image_base: u64,
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
	pub stack_reserve_size: u64,
	pub stack_commit_size: u64,
	pub heap_reserve_size: u64,
	pub heap_commit_size: u64,
	pub loader_flags: u32,
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
	pub flags: u32,
	pub entry_point_token: u32,
	pub resources: DataDirectory,
	pub strong_name_signature_rva: u64,
	#[check_value(|v| *v == 0)]
	pub code_manager_table: u64,
	pub v_table_fixups_rva: u64,
	#[check_value(|v| *v == 0)]
	pub export_address_table_jumps: u64,
	pub managed_native_header: u64,
}

pub mod runtime_flags {
	pub const IL_ONLY: u32 = 0x01;
	pub const REQUIRE_32BIT: u32 = 0x02;
	pub const STRONG_NAME_SIGNED: u32 = 0x08;
	pub const NATIVE_ENTRYPOINT: u32 = 0x10;
	pub const TRACK_DEBUG_DATA: u32 = 0x10000;
}
