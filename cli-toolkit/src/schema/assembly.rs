#[derive(Debug)]
pub struct Assembly {
	name: AssemblyName,
}

#[derive(Debug)]
pub struct AssemblyName {
	name: String,
	public_key: Vec<u8>,
	version: (u16, u16, u16, u16),
}

impl Assembly {
	pub fn name(&self) -> &AssemblyName {
		&self.name
	}
}

impl AssemblyName {
	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn public_key(&self) -> &[u8] {
		&self.public_key
	}

	pub fn version(&self) -> (u16, u16, u16, u16) {
		self.version
	}
}

#[cfg(feature = "read")]
mod read {
	use std::path::Path;
	use crate::raw::*;
	use crate::schema::*;
	use crate::raw::Error::UnexpectedEndOfStream;

	impl crate::schema::Assembly {
		pub fn from_file(path: impl AsRef<Path>) -> Result<Self, Error> {
			let buffer = std::fs::read(path).or(Err(UnexpectedEndOfStream))?;
			Self::try_from(buffer.as_slice())
		}
	}

	impl TryFrom<&[u8]> for crate::schema::Assembly {
		type Error = Error;

		fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
			let raw = crate::raw::Assembly::try_from(bytes)?;
			let blobs = raw.get_heap::<BlobHeap>()?.ok_or(Error::InvalidData)?;
			let tables = raw.get_heap::<TableHeap>()?.ok_or(Error::InvalidData)?;
			let strings = raw.get_heap::<StringHeap>()?.ok_or(Error::InvalidData)?;

			let name = {
				let assembly_table = tables.get_table::<AssemblyTable>()?.ok_or(Error::InvalidData)?;
				let assembly_table = assembly_table.iter().next().ok_or(Error::UnexpectedEndOfStream)??;
				AssemblyName {
					name: strings.get_string(assembly_table.name()).into(),
					public_key: blobs.get_blob(assembly_table.public_key())?.into(),
					version: (
						assembly_table.major_version(),
						assembly_table.minor_version(),
						assembly_table.build_number(),
						assembly_table.revision_number(),
					),
				}
			};

			Ok(Self { name })
		}
	}
}
