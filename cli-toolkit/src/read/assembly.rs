use crate::raw::{
	AlignedBuffer, AssemblyRefTable, AssemblyTable, BlobHeap, CodedIndexKind, MetadataHeap, MetadataTable,
	MetadataTableImpl, StringHeap, TableHeap, TableIndex, TypeDefTable, TypeRefTable,
};
use crate::schema::{Assembly, AssemblyName, AssemblyRef, AssemblyVersion, Context, Type};
use lazy_static::lazy_static;
use std::iter::repeat_with;
use crate::read::Error;
use std::path::PathBuf;
use crate::raw;

pub(crate) struct AssemblyReader<'l> {
	bytes: AlignedBuffer<'l>,
	assembly: &'l mut Assembly<'l>,
}

impl<'l> Assembly<'l> {
	pub(crate) fn default() -> Self {
		lazy_static! {
			static ref EMPTY_CTX: Context<'static> = Context::default();
		}

		Self {
			ctx: &EMPTY_CTX,
			name: AssemblyName {
				flags: 0,
				public_key: vec![],
				name: "".to_string(),
				culture: "".to_string(),
				version: AssemblyVersion {
					major: 0,
					minor: 0,
					build: 0,
					revision: 0,
				},
			},

			types: vec![],
			type_refs: vec![],

			fields: vec![],
			dependencies: vec![],
		}
	}

	pub(crate) fn read(assembly: &'l mut Assembly<'l>, bytes: AlignedBuffer<'l>) -> Result<AssemblyReader<'l>, Error> {
		Ok(AssemblyReader { bytes, assembly })
	}
}

impl<'l> AssemblyReader<'l> {
	pub(crate) fn get_ident(&self) -> Result<String, Error> {
		let raw = crate::raw::Assembly::try_from(self.bytes.as_ref())?;
		let tables = raw
			.get_heap::<TableHeap>()?
			.ok_or(Error::MissingMetadataHeap(TableHeap::cli_identifier()))?;
		let strings = raw
			.get_heap::<StringHeap>()?
			.ok_or(Error::MissingMetadataHeap(StringHeap::cli_identifier()))?;

		let def = tables
			.get_table::<AssemblyTable>()?
			.ok_or(Error::MissingMetadataTable(AssemblyTable::cli_identifier()))?
			.get(TableIndex(1))?;

		let major = def.major_version();
		let minor = def.minor_version();
		let build = def.build_number();
		let revision = def.revision_number();
		let name = strings.get_string(def.name()).to_string();
		let culture = strings.get_string(def.culture()).to_string();

		Ok(format!("{} {} {}.{}.{}.{}", name, culture, major, minor, build, revision))
	}

	pub(crate) fn read(mut self) -> Result<(), Error> {
		let bytes = std::mem::take(&mut self.bytes);
		let raw = crate::raw::Assembly::try_from(bytes.as_ref())?;

		let blobs = raw
			.get_heap::<BlobHeap>()?
			.ok_or(Error::MissingMetadataHeap(BlobHeap::cli_identifier()))?;
		let tables = raw
			.get_heap::<TableHeap>()?
			.ok_or(Error::MissingMetadataHeap(TableHeap::cli_identifier()))?;
		let strings = raw
			.get_heap::<StringHeap>()?
			.ok_or(Error::MissingMetadataHeap(StringHeap::cli_identifier()))?;

		self.read_assembly_name(blobs, strings, tables)?;
		self.read_assembly_refs(blobs, strings, tables)?;
		self.read_assembly_type_refs(strings, tables)?;
		self.read_assembly_types(blobs, strings, tables)?;
		Ok(())
	}

	fn read_assembly_name(&mut self, blobs: BlobHeap, strings: StringHeap, tables: TableHeap) -> Result<(), Error> {
		let def = tables
			.get_table::<AssemblyTable>()?
			.ok_or(Error::MissingMetadataTable(AssemblyTable::cli_identifier()))?
			.get(TableIndex(1))?;

		let assembly_name = &mut self.assembly.name;
		let assembly_version = &mut assembly_name.version;

		assembly_name.flags = def.flags();
		assembly_name.name = strings.get_string(def.name()).to_string();
		assembly_name.culture = strings.get_string(def.culture()).to_string();
		assembly_name.public_key = blobs.get_blob(def.public_key())?.to_vec();

		assembly_version.major = def.major_version();
		assembly_version.minor = def.minor_version();
		assembly_version.build = def.build_number();
		assembly_version.revision = def.revision_number();

		Ok(())
	}

	fn read_assembly_refs(&mut self, blobs: BlobHeap, strings: StringHeap, tables: TableHeap) -> Result<(), Error> {
		let table = match tables.get_table::<AssemblyRefTable>()? {
			Some(table) => table,
			None => return Ok(()),
		};

		self.assembly.dependencies = Vec::with_capacity(table.len());
		for ass_ref in table.iter() {
			let ass_ref = ass_ref?;

			let name = strings.get_string(ass_ref.name()).to_string();
			let culture = strings.get_string(ass_ref.culture()).to_string();
			let version = AssemblyVersion {
				major: ass_ref.major_version(),
				minor: ass_ref.minor_version(),
				build: ass_ref.build_number(),
				revision: ass_ref.revision_number(),
			};

			self.assembly.dependencies.push(AssemblyRef {
				flags: ass_ref.flags(),
				public_key: blobs.get_blob(ass_ref.public_key())?.to_vec(),
				hash_value: blobs.get_blob(ass_ref.public_key())?.to_vec(),
				ident_key: format! {
					"{} {} {}.{}.{}.{}",
					name, culture,
					ass_ref.major_version(),
					ass_ref.minor_version(),
					ass_ref.build_number(),
					ass_ref.revision_number(),
				},
				name,
				culture,
				version,
			})
		}

		Ok(())
	}

	fn read_assembly_type_refs(&mut self, strings: StringHeap, tables: TableHeap) -> Result<(), Error> {
		let table = match tables.get_table::<TypeRefTable>()? {
			Some(table) => table,
			None => return Ok(()),
		};

		self.assembly.type_refs = Vec::with_capacity(table.len());
		for ty in table.iter() {
			let ty = ty?;
			let name = strings.get_string(ty.type_name()).to_string();
			let namespace = strings.get_string(ty.type_namespace()).to_string();
			let token = ty
				.resolution_scope()
				.decode(CodedIndexKind::ResolutionScope)
				.ok_or(raw::Error::InvalidData(Some("Invalid resolution scope")))?;

			self.assembly.type_refs.push((token, namespace, name))
		}

		Ok(())
	}

	fn read_assembly_types(&mut self, blobs: BlobHeap, strings: StringHeap, tables: TableHeap) -> Result<(), Error> {
		let table = match tables.get_table::<TypeDefTable>()? {
			Some(table) => table,
			None => return Ok(()),
		};

		self.assembly.types = repeat_with(Type::default).take(table.len()).collect();

		let types = unsafe {
			let ptr = self.assembly.types.as_mut_ptr() as *mut Type<'l>;
			(0..self.assembly.types.len()).map(move |i| std::mem::transmute(&mut *ptr.add(i)))
		};

		let mut next_def = table.iter().skip(1);
		let iter = types.zip(table.iter()).enumerate();

		//Initialize types
		for (index, (ty, def)) in iter.clone() {
			let def = def?;
			let mut reader = Type::read(index, ty, def, None, blobs, tables, strings, self.assembly);
			reader.initialize()?;
		}

		//Populate types
		for (index, (ty, def)) in iter.clone() {
			let def = def?;
			let mut reader = Type::read(index, ty, def, None, blobs, tables, strings, self.assembly);
			reader.populate()?;
		}

		Ok(())
	}
}
