use crate::metadata::{MetadataIndex, MetadataIndexSize};
use crate::tables::assembly_flags::AssemblyFlags;
use crate::{ParsingError, ZeroCopyReader};
use crate::tables::TableKind::Param;
use crate::assembly::Assembly;
use std::any::{Any, TypeId};
use std::ops::Deref;
use strum::EnumIter;

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumIter)]
pub enum TableKind {
    Module = 0x00,
    TypeRef = 0x01,
    TypeDef = 0x02,
    FieldPtr = 0x03,
    Field = 0x04,
    MethodPtr = 0x05,
    Method = 0x06,
    ParamPtr = 0x07,
    Param = 0x08,
    InterfaceImpl = 0x09,
    MemberRef = 0x0a,
    Constant = 0x0b,
    CustomAttribute = 0x0c,
    FieldMarshal = 0x0d,
    DeclSecurity = 0x0e,
    ClassLayout = 0x0f,
    FieldLayout = 0x10,
    StandAloneSig = 0x11,
    EventMap = 0x12,
    EventPtr = 0x13,
    Event = 0x14,
    PropertyMap = 0x15,
    PropertyPtr = 0x16,
    Property = 0x17,
    MethodSemantics = 0x18,
    MethodImpl = 0x19,
    ModuleRef = 0x1a,
    TypeSpec = 0x1b,
    ImplMap = 0x1c,
    FieldRVA = 0x1d,
    EncLog = 0x1e,
    EncMap = 0x1f,
    Assembly = 0x20,
    AssemblyProcessor = 0x21,
    AssemblyOS = 0x22,
    AssemblyRef = 0x23,
    AssemblyRefProcessor = 0x24,
    AssemblyRefOS = 0x25,
    File = 0x26,
    ExportedType = 0x27,
    ManifestResource = 0x28,
    NestedClass = 0x29,
    GenericParam = 0x2a,
    MethodSpec = 0x2b,
    GenericParamConstraint = 0x2c,

    Document = 0x30,
    MethodDebugInformation = 0x31,
    LocalScope = 0x32,
    LocalVariable = 0x33,
    LocalConstant = 0x34,
    ImportScope = 0x35,
    StateMachineMethod = 0x36,
    CustomDebugInformation = 0x37,
}

pub trait MetadataTable<'l> {
    fn kind(&self) -> TableKind;
    fn static_kind() -> TableKind
    where
        Self: Sized;
}

pub struct GenericMetadataTable<'l> {
    table: Box<dyn MetadataTable<'l> + 'l>,
}

impl<'l> GenericMetadataTable<'l> {
    pub fn downcast<T: MetadataTable<'l>>(&self) -> Option<&T> {
        match self.table.kind() == T::static_kind() {
            false => None,
            true => unsafe {
                let ptr = self.table.deref() as *const _ as *const T;
                Some(&*ptr)
            },
        }
    }
}

impl<'l, T: MetadataTable<'l> + 'l> From<T> for GenericMetadataTable<'l> {
    fn from(value: T) -> Self {
        Self {
            table: Box::new(value),
        }
    }
}

impl<'l> Deref for GenericMetadataTable<'l> {
    type Target = dyn MetadataTable<'l> + 'l;

    fn deref(&self) -> &Self::Target {
        self.table.deref()
    }
}

pub(crate) struct DummyTable {
	kind: TableKind,
}

impl DummyTable {
	pub(crate) fn new(kind: TableKind) -> Self {
		Self { kind }
	}
}

impl <'l> MetadataTable<'l> for DummyTable {
	fn kind(&self) -> TableKind {
		self.kind
	}

	fn static_kind() -> TableKind where Self: Sized {
		unreachable!()
	}
}

pub struct ModuleTable<'l> {
    bytes: &'l [u8],
    guid_idx_size: MetadataIndexSize,
    string_idx_size: MetadataIndexSize,
}

impl<'l> ModuleTable<'l> {
    pub(crate) fn new(
        bytes: &'l [u8],
        guid_idx_size: MetadataIndexSize,
        string_idx_size: MetadataIndexSize,
    ) -> Self {
        Self {
            bytes,
            guid_idx_size,
            string_idx_size,
        }
    }

    pub fn rows(&self) -> ModuleTableIterator {
        ModuleTableIterator {
            bytes: self.bytes,
            reader: ZeroCopyReader::new(self.bytes),
            guid_idx_size: self.guid_idx_size,
            string_idx_size: self.string_idx_size,
        }
    }
}

impl<'l> MetadataTable<'l> for ModuleTable<'l> {
    fn kind(&self) -> TableKind {
        TableKind::Module
    }

    fn static_kind() -> TableKind
    where
        Self: Sized,
    {
        TableKind::Module
    }
}

#[derive(Debug)]
pub struct Module {
    generation: u16,
    name: MetadataIndex,
    module_version_id: MetadataIndex,
    enc_id: MetadataIndex,
    enc_base_id: MetadataIndex,
}

impl Module {
	pub(crate) fn row_size(guid_idx_size: MetadataIndexSize, string_idx_size: MetadataIndexSize) -> usize {
        2 + string_idx_size as usize + guid_idx_size as usize * 3
    }

    pub fn generation(&self) -> u16 {
        self.generation
    }

    pub fn name(&self) -> MetadataIndex {
        self.name
    }

    pub fn module_version_id(&self) -> MetadataIndex {
        self.module_version_id
    }

    pub fn enc_id(&self) -> MetadataIndex {
        self.enc_id
    }

    pub fn enc_base_id(&self) -> MetadataIndex {
        self.enc_base_id
    }
}

pub struct ModuleTableIterator<'l> {
    bytes: &'l [u8],
    reader: ZeroCopyReader<'l>,
    guid_idx_size: MetadataIndexSize,
    string_idx_size: MetadataIndexSize,
}

impl Iterator for ModuleTableIterator<'_> {
    type Item = Module;

    fn next(&mut self) -> Option<Self::Item> {
        Some(Module {
            generation: *self.reader.read::<u16>().ok()?,
            name: self.reader.read_index(self.string_idx_size).ok()?,
            module_version_id: self.reader.read_index(self.guid_idx_size).ok()?,
            enc_id: self.reader.read_index(self.guid_idx_size).ok()?,
            enc_base_id: self.reader.read_index(self.guid_idx_size).ok()?,
        })
    }
}

pub struct TypeRefTable<'l> {
	bytes: &'l [u8],
	res_idx_size: MetadataIndexSize,
	str_idx_size: MetadataIndexSize,
}

impl <'l> TypeRefTable<'l> {
	pub(crate) fn new(bytes: &'l [u8], res_idx_size: MetadataIndexSize, str_idx_size: MetadataIndexSize) -> Self {
		Self { bytes, res_idx_size, str_idx_size }
	}
	
	pub fn rows(&'l self) -> TypeRefIterator<'l> {
		TypeRefIterator {
			reader: ZeroCopyReader::new(self.bytes),
			res_idx_size: self.res_idx_size,
			str_idx_size: self.str_idx_size,
		}
	}
}

impl <'l> MetadataTable<'l> for TypeRefTable<'l> {
	fn kind(&self) -> TableKind {
		TableKind::TypeRef
	}

	fn static_kind() -> TableKind where Self: Sized {
		TableKind::TypeRef
	}
}

#[derive(Debug, Clone)]
pub struct TypeRef {
	resolution_scope: MetadataIndex,
	type_name: MetadataIndex,
	type_namespace: MetadataIndex,
}

impl TypeRef {
	pub(crate) fn row_size(res_idx_size: MetadataIndexSize, str_idx_size: MetadataIndexSize) -> usize {
		res_idx_size as usize + str_idx_size as usize * 2
	}
	
	pub fn resolution_scope(&self) -> MetadataIndex {
		self.resolution_scope
	}
	
	pub fn name(&self) -> MetadataIndex {
		self.type_name
	}
	
	pub fn namespace(&self) -> MetadataIndex {
		self.type_namespace
	}
}

pub struct TypeRefIterator<'l> {
	reader: ZeroCopyReader<'l>,
	res_idx_size: MetadataIndexSize,
	str_idx_size: MetadataIndexSize,
}

impl Iterator for TypeRefIterator<'_> {
	type Item = TypeRef;

	fn next(&mut self) -> Option<Self::Item> {
		let resolution_scope = self.reader.read_index(self.res_idx_size).ok()?;
		let type_name = self.reader.read_index(self.str_idx_size).ok()?;
		let type_namespace = self.reader.read_index(self.str_idx_size).ok()?;
		Some(TypeRef { resolution_scope, type_name, type_namespace})
	}
}

#[derive(Debug)]
pub struct AssemblyTable {
	hash_algorithm: AssemblyHashAlgorithm,
	major_version: u16,
	minor_version: u16,
	build_number: u16,
	revision_number: u16,
	flags: AssemblyFlags,
	public_key: MetadataIndex,
	name: MetadataIndex,
	culture: MetadataIndex,
}

impl AssemblyTable {
	pub(crate) fn new(
		bytes: &[u8], 
		blob_idx_size: MetadataIndexSize, 
		string_idx_size: MetadataIndexSize
	) -> Result<Self, ParsingError> {
		let mut reader = ZeroCopyReader::new(bytes);
		Ok(
			Self {
				hash_algorithm: *reader.read::<AssemblyHashAlgorithm>()?,
				major_version: *reader.read::<u16>()?,
				minor_version: *reader.read::<u16>()?,
				build_number: *reader.read::<u16>()?,
				revision_number: *reader.read::<u16>()?,
				flags: *reader.read::<AssemblyFlags>()?,
				public_key: reader.read_index(blob_idx_size)?,
				name: reader.read_index(string_idx_size)?,
				culture: reader.read_index(string_idx_size)?,
			}
		)
	}
}

impl MetadataTable<'_> for AssemblyTable {
	fn kind(&self) -> TableKind {
		TableKind::Assembly
	}

	fn static_kind() -> TableKind where Self: Sized {
		TableKind::Assembly
	}
}

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum AssemblyHashAlgorithm {
	None = 0x0000,
	MD5 = 0x8003,
	SHA1 = 0x8004,
}

pub mod assembly_flags {
	pub type AssemblyFlags = u32;
	pub const PUBLIC_KEY: AssemblyFlags = 0x0001;
	pub const RETARGETABLE: AssemblyFlags = 0x0100;
	pub const DISABLE_JIT_COMPILE_OPTIMIZER: AssemblyFlags = 0x4000;
	pub const ENABLE_JIT_COMPILE_TRACKING: AssemblyFlags = 0x8000;
}