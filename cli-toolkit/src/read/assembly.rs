use crate::raw::{
	AlignedBuffer, AssemblyRefTable, AssemblyTable, BlobHeap, CodedIndexKind, MetadataHeap, MetadataTable,
	MetadataTableImpl, StringHeap, TableHeap, TableIndex, TypeDefTable, TypeRefTable,
};
use crate::schema::{Assembly, AssemblyName, AssemblyRef, AssemblyVersion, Context, Type};
use crate::utilities::get_mut_unchecked;
use std::marker::PhantomData;
use lazy_static::lazy_static;
use std::iter::repeat_with;
use std::rc::{Rc, Weak};
use crate::read::Error;
use std::path::PathBuf;
use std::ptr::null;
use crate::raw;

pub(crate) struct AssemblyReader<'l> {
	bytes: AlignedBuffer<'l>,

	blobs: BlobHeap<'l>,
	tables: TableHeap<'l>,
	strings: StringHeap<'l>,
	raw_assembly: raw::Assembly<'l>,
}

impl Assembly {
	pub(crate) fn default() -> Self {
		Self {
			ctx: Weak::new(),
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

			types: Rc::new([]),
			type_refs: vec![],
			dependencies: vec![],
		}
	}
}

impl<'l> AssemblyReader<'l> {
	pub(super) fn new(bytes: AlignedBuffer<'l>) -> Result<Self, Error> {
		let raw_assembly = raw::Assembly::try_from(unsafe { std::mem::transmute::<_, &'l [u8]>(bytes.as_ref()) })?;

		let blobs = raw_assembly
			.get_heap::<BlobHeap>()?
			.ok_or(Error::MissingMetadataHeap(BlobHeap::cli_identifier()))?;

		let tables = raw_assembly
			.get_heap::<TableHeap>()?
			.ok_or(Error::MissingMetadataHeap(TableHeap::cli_identifier()))?;

		let strings = raw_assembly
			.get_heap::<StringHeap>()?
			.ok_or(Error::MissingMetadataHeap(StringHeap::cli_identifier()))?;

		Ok(Self {
			bytes,
			blobs,
			tables,
			strings,
			raw_assembly,
		})
	}

	pub(super) fn get_ident(&self) -> Result<String, Error> {
		let def = self
			.tables
			.get_table::<AssemblyTable>()?
			.ok_or(Error::MissingMetadataTable(AssemblyTable::cli_identifier()))?
			.get(TableIndex(1))?;

		let major = def.major_version();
		let minor = def.minor_version();
		let build = def.build_number();
		let revision = def.revision_number();
		let name = self.strings.get_string(def.name()).to_string();
		let culture = self.strings.get_string(def.culture()).to_string();

		Ok(format!("{} {} {}.{}.{}.{}", name, culture, major, minor, build, revision))
	}

	pub(super) fn read_assembly_definition(&self, mut assembly: Rc<Assembly>) -> Result<Rc<Assembly>, Error> {
		let def = self
			.tables
			.get_table::<AssemblyTable>()?
			.ok_or(Error::MissingMetadataTable(AssemblyTable::cli_identifier()))?
			.get(TableIndex(1))?;

		{
			let assembly = Rc::get_mut(&mut assembly).unwrap();

			let assembly_name = &mut assembly.name;
			let assembly_version = &mut assembly_name.version;

			assembly_name.flags = def.flags();
			assembly_name.name = self.strings.get_string(def.name()).to_string();
			assembly_name.culture = self.strings.get_string(def.culture()).to_string();
			assembly_name.public_key = self.blobs.get_blob(def.public_key())?.to_vec();

			assembly_version.major = def.major_version();
			assembly_version.minor = def.minor_version();
			assembly_version.build = def.build_number();
			assembly_version.revision = def.revision_number();
		}

		Ok(assembly)
	}

	pub(super) fn read_assembly_refs(&self, assembly: &mut Assembly) -> Result<(), Error> {
		let table = match self.tables.get_table::<AssemblyRefTable>()? {
			Some(table) => table,
			None => return Ok(()),
		};

		assembly.dependencies = Vec::with_capacity(table.len());
		for ass_ref in table.iter() {
			let ass_ref = ass_ref?;

			let name = self.strings.get_string(ass_ref.name()).to_string();
			let culture = self.strings.get_string(ass_ref.culture()).to_string();
			let version = AssemblyVersion {
				major: ass_ref.major_version(),
				minor: ass_ref.minor_version(),
				build: ass_ref.build_number(),
				revision: ass_ref.revision_number(),
			};

			assembly.dependencies.push(AssemblyRef {
				flags: ass_ref.flags(),
				public_key: self.blobs.get_blob(ass_ref.public_key())?.to_vec(),
				hash_value: self.blobs.get_blob(ass_ref.public_key())?.to_vec(),
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

	pub(super) fn read_assembly_type_refs(&self, assembly: &mut Assembly) -> Result<(), Error> {
		let table = match self.tables.get_table::<TypeRefTable>()? {
			Some(table) => table,
			None => return Ok(()),
		};

		assembly.type_refs = Vec::with_capacity(table.len());
		for ty in table.iter() {
			let ty = ty?;
			let name = self.strings.get_string(ty.type_name()).to_string();
			let namespace = self.strings.get_string(ty.type_namespace()).to_string();
			let token = ty
				.resolution_scope()
				.decode(CodedIndexKind::ResolutionScope)
				.ok_or(raw::Error::InvalidData(Some("Invalid resolution scope")))?;

			assembly.type_refs.push((token, namespace, name))
		}

		Ok(())
	}

	pub(super) fn read_assembly_types(&self, assembly: Rc<Assembly>) -> Result<(), Error> {
		let table = match self.tables.get_table::<TypeDefTable>()? {
			Some(table) => table,
			None => return Ok(()),
		};

		let mut types = Rc::from_iter(repeat_with(Type::default).take(table.len()));

		for index in 0..table.len() {
			let reader = Type::read(self.blobs, self.tables, self.strings, table.clone(), assembly.clone());
			reader.read_type_definition(index, &mut types);
		}

		for index in 0..table.len() {
			let reader = Type::read(self.blobs, self.tables, self.strings, table.clone(), assembly.clone());
			reader.read_base(index, &mut types);
		}

		let mut_assembly = unsafe { get_mut_unchecked(&assembly) };
		mut_assembly.types = types;

		Ok(())
	}
}
