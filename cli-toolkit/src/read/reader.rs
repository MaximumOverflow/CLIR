use crate::raw::{Assembly as RawAssembly, AssemblyRefTable, AssemblyTable};
use crate::raw::Error as ReadError;
pub use crate::read::Type;
pub use crate::read::*;
use std::path::PathBuf;
use std::fmt::Debug;
use std::rc::Rc;

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
				let assembly = RawAssembly::try_from(bytes.as_slice()).ok()?;

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

				let assembly = match RawAssembly::try_from(bytes.as_slice()) {
					Ok(assembly) => assembly,
					Err(err) => {
						return match err {
							ReadError::InvalidData(_, _) => Err(Error::InvalidAssembly(InvalidAssemblyError::Unknown)),
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

							let name = format! {
								"{} {}",
								strings.get_string(dependency.name()),
								dependency.major_version(),
							};

							dependencies.push(self.read_assembly(&name)?)
						}
					}

					dependencies
				};

				let types = Type::read_all(blobs, tables, strings)?;

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

						types,
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
enum AssemblyEntry {
	NotLoaded(PathBuf),
	Loaded(Rc<Assembly>),
}
