use crate::raw::*;

#[derive(Debug, Clone)]
pub struct MetadataHeader<'l> {
	pub signature: u32,
	pub major_version: u16,
	pub minor_version: u16,
	pub length: u32,
	pub version: &'l str,
	pub flags: u16,
	pub stream_count: u16,

	offset: usize,
	streams: &'l [u8],
	assembly_bytes: &'l [u8],
}

#[repr(C)]
#[derive(Debug)]
pub struct StreamHeader<'l> {
	offset: u32,
	size: u32,
	name: &'l str,
}

impl<'l> MetadataHeader<'l> {
	pub(crate) fn new(assembly_bytes: &'l [u8], offset: usize) -> Result<Self, Error> {
		let mut reader = ByteStream::new(&assembly_bytes[offset..]);
		let signature = reader.read::<u32>()?;
		let major_version = reader.read::<u16>()?;
		let minor_version = reader.read::<u16>()?;
		reader.skip(4)?; //reserved
		let length = reader.read::<u32>()?;
		let version = {
			let version = reader.read_null_terminated_str()?;
			reader.skip(length as usize - version.len() - 1)?;
			version
		};
		let flags = reader.read::<u16>()?;
		let stream_count = reader.read::<u16>()?;
		let streams = {
			let start = reader.position();
			for _ in 0..stream_count {
				reader.skip(8)?;
				let name = reader.read_u8_slice_until(0)?;
				reader.skip((4usize.wrapping_sub(name.len())) % 4)?;
			}

			&reader.bytes()[start..reader.position()]
		};

		Ok(Self {
			signature,
			major_version,
			minor_version,
			length,
			version,
			flags,
			streams,
			stream_count,
			offset,
			assembly_bytes,
		})
	}

	pub(crate) fn get_heap<T: MetadataHeap<'l>>(&self) -> Result<Option<T>, Error> {
		let bytes = self.get_stream_bytes(T::cli_identifier())?;
		Ok(bytes.map(|b| T::new(b)))
	}

	fn stream_headers(&self) -> StreamHeaderIterator {
		StreamHeaderIterator {
			reader: ByteStream::new(self.streams),
		}
	}

	fn get_stream_bytes(&self, name: &str) -> Result<Option<&'l [u8]>, Error> {
		for header in self.stream_headers() {
			let header = header?;
			let start = self.offset + header.offset as usize;

			if header.name == name {
				return Ok(Some(&self.assembly_bytes[start..start + header.size as usize]));
			}
		}

		Ok(None)
	}
}

pub struct StreamHeaderIterator<'l> {
	reader: ByteStream<'l>,
}

impl<'l> Iterator for StreamHeaderIterator<'l> {
	type Item = Result<StreamHeader<'l>, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.reader.remaining() {
			0 => None,
			_ => {
				let offset = match self.reader.read::<u32>() {
					Ok(v) => v,
					Err(e) => return Some(Err(e)),
				};
				let size = match self.reader.read::<u32>() {
					Ok(v) => v,
					Err(e) => return Some(Err(e)),
				};
				let name = match self.reader.read_null_terminated_str() {
					Ok(v) => v,
					Err(e) => return Some(Err(e)),
				};

				//Not sure if this is ok
				if self.reader.skip((4usize.wrapping_sub(name.len() + 1)) % 4).is_err() {
					self.reader.skip(self.reader.remaining()).unwrap();
				}
				Some(Ok(StreamHeader { offset, size, name }))
			}
		}
	}
}
