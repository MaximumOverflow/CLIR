use crate::raw::{Assembly, AssemblyTable, MetadataTable, StringHeap, TableHeap};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use std::io::Read;

#[derive(Debug)]
pub struct AssemblyReader<'l> {
	assemblies: HashMap<String, AssemblyEntry<'l>>,
}

enum AssemblyEntry<'l> {
	Unloaded(PathBuf),
	Loaded { bytes: Vec<u8>, raw_assembly: Assembly<'l> },
}

impl AssemblyReader<'_> {
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
				let assembly = Assembly::try_from(bytes.as_slice()).ok()?;

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

				Some((name, AssemblyEntry::Unloaded(path)))
			});

		Self {
			assemblies: HashMap::from_iter(assemblies),
		}
	}
}

impl Debug for AssemblyEntry<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			AssemblyEntry::Unloaded(path) => {
				write!(f, "Unloaded({:?})", path)
			}
			AssemblyEntry::Loaded { .. } => {
				write!(f, "Loaded")
			}
		}
	}
}
