use crate::__impl_clone_from_byte_stream;
use crate::raw::*;

pub struct Assembly<'l> {
	#[allow(unused)]
	pe_header: PeHeader,
	#[allow(unused)]
	pe_optional_header: PeOptionalHeader,
	#[allow(unused)]
	cli_header: CliHeader,

	bytes: &'l [u8],
	metadata_header: MetadataHeader<'l>,
}

impl<'l> TryFrom<&'l [u8]> for Assembly<'l> {
	type Error = Error;

	fn try_from(bytes: &'l [u8]) -> Result<Self, Self::Error> {
		let mut reader = ByteStream::new(bytes);
		reader.seek(0x3C)?;
		let pe_start = (reader.read::<u32>()? + 4) as usize;
		reader.seek(pe_start)?;

		let pe_header = PeHeader::from_byte_stream(&mut reader)?;
		let pe_optional_header = PeOptionalHeader::from_byte_stream(&mut reader)?;
		let sections = reader.read_slice::<SectionHeader>(pe_header.number_of_sections as usize)?;

		reader.seek(resolve_rva(pe_optional_header.data_directories[14].rva, sections)?)?;
		let cli_header = CliHeader::from_byte_stream(&mut reader)?;

		let metadata_start = resolve_rva(cli_header.metadata.rva, sections)?;
		let metadata_header = MetadataHeader::new(bytes, metadata_start)?;

		Ok(Assembly {
			bytes,
			pe_header,
			pe_optional_header,
			cli_header,
			metadata_header,
		})
	}
}

impl<'l> Assembly<'l> {
	pub fn bytes(&self) -> &'l [u8] {
		self.bytes
	}

	pub fn get_heap<T: MetadataHeap<'l>>(&self) -> Result<Option<T>, Error> {
		self.metadata_header.get_heap()
	}
}

#[repr(C)]
#[derive(Debug, Clone)]
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

fn resolve_rva(rva: u32, sections: &[SectionHeader]) -> Result<usize, Error> {
	let section = sections
		.iter()
		.find(|s| rva >= s.virtual_address && rva < (s.virtual_address + s.size_of_raw_data))
		.ok_or(Error::OffsetOutOfBounds)?;

	Ok((rva - section.virtual_address + section.pointer_to_raw_data) as usize)
}

__impl_clone_from_byte_stream!(CliHeader);
