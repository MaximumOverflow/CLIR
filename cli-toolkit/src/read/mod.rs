mod assembly;
mod reader;
mod types;

pub use types::*;
pub use reader::*;
pub use assembly::*;
use crate::raw::{MetadataHeap, MetadataTable, TableHeap, StringHeap, BlobHeap, Assembly as RawAssembly};

#[derive(Debug)]
pub enum Error {
	IOError(std::io::Error),
	AssemblyNotFound(String),
	ReadError(crate::raw::Error),
	InvalidAssembly(InvalidAssemblyError),
}

#[derive(Debug)]
pub enum InvalidAssemblyError {
	Unknown,
	MissingMetadataHeap,
	MissingMetadataTable,
	MissingMetadataTableField,
}

fn get_heap<'l, T: MetadataHeap<'l>>(assembly: &'l RawAssembly) -> Result<T, Error> {
	match assembly.get_heap::<T>() {
		Err(err) => Err(Error::ReadError(err)),
		Ok(heap) => match heap {
			None => Err(Error::InvalidAssembly(InvalidAssemblyError::MissingMetadataHeap)),
			Some(heap) => Ok(heap),
		},
	}
}

fn get_table<'l, T: MetadataTable<'l>>(tables: &'l TableHeap<'l>) -> Result<T, Error> {
	match tables.get_table::<T>() {
		Err(err) => Err(Error::ReadError(err)),
		Ok(table) => match table {
			None => Err(Error::InvalidAssembly(InvalidAssemblyError::MissingMetadataTable)),
			Some(table) => Ok(table),
		},
	}
}

fn try_get_table<'l, T: MetadataTable<'l>>(tables: &'l TableHeap<'l>) -> Result<Option<T>, Error> {
	match tables.get_table::<T>() {
		Err(err) => Err(Error::ReadError(err)),
		Ok(table) => match table {
			None => Ok(None),
			Some(table) => Ok(Some(table)),
		},
	}
}
