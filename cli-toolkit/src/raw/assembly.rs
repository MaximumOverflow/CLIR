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
		let dos_header = DosHeader::from_byte_stream(&mut reader)?;

		let pe_start = dos_header.lfanew() as usize;
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

fn resolve_rva(rva: u32, sections: &[SectionHeader]) -> Result<usize, Error> {
	let section = sections
		.iter()
		.find(|s| rva >= s.virtual_address && rva < (s.virtual_address + s.size_of_raw_data))
		.ok_or(Error::OffsetOutOfBounds)?;

	Ok((rva - section.virtual_address + section.pointer_to_raw_data) as usize)
}
