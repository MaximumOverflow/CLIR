use crate::raw::{
	AssemblyRef, AssemblyRefTable, AssemblyTable, BlobHeap, MetadataHeap, MetadataTable, StringHeap, TableHeap,
};
pub use crate::read::*;
use std::path::PathBuf;
use std::fmt::{Debug, Formatter};
use std::io::Read;
use std::rc::Rc;
use crate::raw;

#[derive(Debug)]
pub struct AssemblyReader {
	assemblies: Vec<(String, AssemblyEntry)>,
}

impl AssemblyReader {
	pub fn new(include_paths: Vec<PathBuf>) -> Self {
		let assemblies = include_paths
			.into_iter()
			.map(|path| std::fs::read_dir(path))
			.flatten()
			.flatten()
			.flatten()
			.filter_map(|entry| {
				let path = entry.path();
				let extension = path.extension().map(|e| e.to_str())??;

				if extension != "dll" {
					return None;
				}

				let bytes = std::fs::read(&path).ok()?;
				let assembly = raw::Assembly::try_from(bytes.as_slice()).ok()?;

				let tables = assembly.get_heap::<TableHeap>().ok()??;
				let strings = assembly.get_heap::<StringHeap>().ok()??;
				let assembly = tables.get_table::<AssemblyTable>().ok()??.iter().next()?.ok()?;

				let name = format!(
					"{} {}.{}.{}.{}",
					strings.get_string(assembly.name()),
					assembly.major_version(),
					assembly.minor_version(),
					assembly.build_number(),
					assembly.revision_number(),
				);

				Some((name, AssemblyEntry::NotLoaded(path)))
			})
			.collect();

		Self { assemblies }
	}

	pub fn read_assembly(&mut self, name: &str) -> Result<Rc<Assembly>, Error> {
		let entry = self
			.assemblies
			.iter()
			.enumerate()
			.find_map(|(idx, (key, entry))| match key.starts_with(name) {
				true => Some((idx, entry)),
				false => None,
			});

		let (index, entry) = match entry {
			None => return Err(Error::AssemblyNotFound(name.into())),
			Some(entry) => entry,
		};

		match entry {
			AssemblyEntry::Loaded(assembly) => Ok(assembly.clone()),

			AssemblyEntry::NotLoaded(path) => {
				let bytes = match std::fs::read(path) {
					Err(err) => return Err(Error::IOError(err)),
					Ok(bytes) => bytes,
				};

				let assembly = match raw::Assembly::try_from(bytes.as_slice()) {
					Ok(assembly) => assembly,
					Err(err) => {
						return match err {
							raw::Error::InvalidData(_, _) => Err(Error::InvalidAssembly(InvalidAssemblyError::Unknown)),
							error => Err(Error::ReadError(error)),
						}
					}
				};

				let blobs = get_heap::<BlobHeap>(&assembly)?;
				let tables = get_heap::<TableHeap>(&assembly)?;
				let strings = get_heap::<StringHeap>(&assembly)?;

				let dependencies = {
					let mut dependencies = vec![];
					let ref_table = try_get_table::<AssemblyRefTable>(&tables)?;

					if let Some(deps) = ref_table {
						for dependency in deps.iter() {
							let dependency = match dependency {
								Ok(dep) => dep,
								Err(err) => return Err(Error::ReadError(err)),
							};

							let name = format!("{} {}",
								strings.get_string(dependency.name()),
								dependency.major_version(),
							);

							dependencies.push(self.read_assembly(&name)?)
						}
					}

					dependencies
				};

				let assembly = {
					let def = match get_table::<AssemblyTable>(&tables)?.iter().next() {
						None => return Err(Error::InvalidAssembly(InvalidAssemblyError::MissingMetadataTableField)),
						Some(assembly) => match assembly {
							Ok(assembly) => assembly,
							Err(err) => return Err(Error::ReadError(err)),
						},
					};

					Rc::new(Assembly {
						name: strings.get_string(def.name()).into(),
						culture: strings.get_string(def.culture()).into(),
						version: AssemblyVersion {
							major: def.major_version(),
							minor: def.minor_version(),
							build: def.build_number(),
							revision: def.revision_number(),
						},
						dependencies,
					})
				};

				self.assemblies[index].1 = AssemblyEntry::Loaded(assembly.clone());
				Ok(assembly)
			}
		}
	}
}

#[derive(Debug)]
pub enum Error {
	ReadError(raw::Error),
	IOError(std::io::Error),
	AssemblyNotFound(String),
	InvalidAssembly(InvalidAssemblyError),
}

#[derive(Debug)]
pub enum InvalidAssemblyError {
	Unknown,
	MissingMetadataHeap,
	MissingMetadataTable,
	MissingMetadataTableField,
}

enum AssemblyEntry {
	NotLoaded(PathBuf),
	Loaded(Rc<Assembly>),
}

impl Debug for AssemblyEntry {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			AssemblyEntry::NotLoaded(path) => write!(f, "NotLoaded({:?})", path),
			AssemblyEntry::Loaded(ass) => write!(f, "Loaded({:?})", ass.name)
		}
	}
}

fn get_heap<'l, T: MetadataHeap<'l>>(assembly: &'l raw::Assembly) -> Result<T, Error> {
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
