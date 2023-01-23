use crate::raw::TableKind;

mod assembly;
mod context;
mod types;

#[derive(Debug)]
pub enum Error {
	IOError(std::io::Error),
	ReadError(crate::raw::Error),
	MissingMetadataTable(TableKind),
	MissingMetadataHeap(&'static str),
}

impl From<std::io::Error> for Error {
	fn from(value: std::io::Error) -> Self {
		Self::IOError(value)
	}
}

impl From<crate::raw::Error> for Error {
	fn from(value: crate::raw::Error) -> Self {
		Self::ReadError(value)
	}
}
