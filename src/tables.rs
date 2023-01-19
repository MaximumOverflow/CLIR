use crate::metadata::{MetadataIndex, MetadataIndexSize};
use crate::tables::assembly_flags::AssemblyFlags;
use crate::tables::type_attributes::TypeAttributes;
use crate::{ParsingError, ZeroCopyReader};
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
		Self { table: Box::new(value) }
	}
}

impl<'l> Deref for GenericMetadataTable<'l> {
	type Target = dyn MetadataTable<'l> + 'l;

	fn deref(&self) -> &Self::Target {
		self.table.deref()
	}
}

pub struct ModuleTable<'l> {
	bytes: &'l [u8],
	guid_idx_size: MetadataIndexSize,
	string_idx_size: MetadataIndexSize,
}

impl<'l> ModuleTable<'l> {
	pub(crate) fn new(bytes: &'l [u8], guid_idx_size: MetadataIndexSize, string_idx_size: MetadataIndexSize) -> Self {
		Self {
			bytes,
			guid_idx_size,
			string_idx_size,
		}
	}

	pub fn rows(&self) -> ModuleTableIterator {
		ModuleTableIterator {
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
	reader: ZeroCopyReader<'l>,
	guid_idx_size: MetadataIndexSize,
	string_idx_size: MetadataIndexSize,
}

impl Iterator for ModuleTableIterator<'_> {
	type Item = Result<Module, ParsingError>;

	fn next(&mut self) -> Option<Self::Item> {
		fn parse(this: &mut ModuleTableIterator) -> Result<Module, ParsingError> {
			Ok(Module {
				generation: *this.reader.read::<u16>()?,
				name: this.reader.read_index(this.string_idx_size)?,
				module_version_id: this.reader.read_index(this.guid_idx_size)?,
				enc_id: this.reader.read_index(this.guid_idx_size)?,
				enc_base_id: this.reader.read_index(this.guid_idx_size)?,
			})
		}

		match self.reader.remaining() {
			0 => None,
			_ => Some(parse(self)),
		}
	}
}

pub struct TypeRefTable<'l> {
	bytes: &'l [u8],
	res_idx_size: MetadataIndexSize,
	str_idx_size: MetadataIndexSize,
}

impl<'l> TypeRefTable<'l> {
	pub(crate) fn new(bytes: &'l [u8], res_idx_size: MetadataIndexSize, str_idx_size: MetadataIndexSize) -> Self {
		Self {
			bytes,
			res_idx_size,
			str_idx_size,
		}
	}

	pub fn rows(&'l self) -> TypeRefIterator<'l> {
		TypeRefIterator {
			reader: ZeroCopyReader::new(self.bytes),
			res_idx_size: self.res_idx_size,
			str_idx_size: self.str_idx_size,
		}
	}
}

impl<'l> MetadataTable<'l> for TypeRefTable<'l> {
	fn kind(&self) -> TableKind {
		TableKind::TypeRef
	}

	fn static_kind() -> TableKind
	where
		Self: Sized,
	{
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
	type Item = Result<TypeRef, ParsingError>;

	fn next(&mut self) -> Option<Self::Item> {
		fn parse(this: &mut TypeRefIterator) -> Result<TypeRef, ParsingError> {
			Ok(TypeRef {
				resolution_scope: this.reader.read_index(this.res_idx_size)?,
				type_name: this.reader.read_index(this.str_idx_size)?,
				type_namespace: this.reader.read_index(this.str_idx_size)?,
			})
		}

		match self.reader.remaining() {
			0 => None,
			_ => Some(parse(self)),
		}
	}
}

pub struct TypeDefTable<'l> {
	bytes: &'l [u8],
	str_idx_size: MetadataIndexSize,
	ext_idx_size: MetadataIndexSize,
	fld_idx_size: MetadataIndexSize,
	mtd_idx_size: MetadataIndexSize,
}

impl<'l> TypeDefTable<'l> {
	pub fn new(
		bytes: &'l [u8],
		str_idx_size: MetadataIndexSize,
		ext_idx_size: MetadataIndexSize,
		fld_idx_size: MetadataIndexSize,
		mtd_idx_size: MetadataIndexSize,
	) -> Self {
		Self {
			bytes,
			str_idx_size,
			ext_idx_size,
			fld_idx_size,
			mtd_idx_size,
		}
	}

	pub fn rows(&'l self) -> TypeDefIterator<'l> {
		TypeDefIterator {
			reader: ZeroCopyReader::new(self.bytes),
			str_idx_size: self.str_idx_size,
			ext_idx_size: self.ext_idx_size,
			fld_idx_size: self.fld_idx_size,
			mtd_idx_size: self.mtd_idx_size,
		}
	}
}

impl<'l> MetadataTable<'l> for TypeDefTable<'l> {
	fn kind(&self) -> TableKind {
		TableKind::TypeDef
	}

	fn static_kind() -> TableKind
	where
		Self: Sized,
	{
		TableKind::TypeDef
	}
}

#[derive(Debug, Clone)]
pub struct TypeDef {
	flags: TypeAttributes,
	name: MetadataIndex,
	namespace: MetadataIndex,
	extends: MetadataIndex,
	fields: MetadataIndex,
	methods: MetadataIndex,
}

impl TypeDef {
	pub(crate) fn row_size(
		str_idx_size: MetadataIndexSize,
		ext_idx_size: MetadataIndexSize,
		fld_idx_size: MetadataIndexSize,
		mtd_idx_size: MetadataIndexSize,
	) -> usize {
		4 + str_idx_size as usize * 2 + ext_idx_size as usize + fld_idx_size as usize + mtd_idx_size as usize
	}

	pub fn flags(&self) -> TypeAttributes {
		self.flags
	}

	pub fn name(&self) -> MetadataIndex {
		self.name
	}

	pub fn namespace(&self) -> MetadataIndex {
		self.namespace
	}

	pub fn extends(&self) -> MetadataIndex {
		self.extends
	}

	pub fn fields(&self) -> MetadataIndex {
		self.fields
	}

	pub fn methods(&self) -> MetadataIndex {
		self.methods
	}
}

pub struct TypeDefIterator<'l> {
	reader: ZeroCopyReader<'l>,
	str_idx_size: MetadataIndexSize,
	ext_idx_size: MetadataIndexSize,
	fld_idx_size: MetadataIndexSize,
	mtd_idx_size: MetadataIndexSize,
}

impl Iterator for TypeDefIterator<'_> {
	type Item = Result<TypeDef, ParsingError>;

	fn next(&mut self) -> Option<Self::Item> {
		fn parse(this: &mut TypeDefIterator) -> Result<TypeDef, ParsingError> {
			Ok(TypeDef {
				flags: this.reader.read_unaligned::<TypeAttributes>()?,
				name: this.reader.read_index(this.str_idx_size)?,
				namespace: this.reader.read_index(this.str_idx_size)?,
				extends: this.reader.read_index(this.ext_idx_size)?,
				fields: this.reader.read_index(this.fld_idx_size)?,
				methods: this.reader.read_index(this.mtd_idx_size)?,
			})
		}

		match self.reader.remaining() {
			0 => None,
			_ => Some(parse(self)),
		}
	}
}

pub mod type_attributes {
	pub type TypeAttributes = u32;

	//Visibility attributes
	pub const VISIBILITY_MASK: TypeAttributes = 0x00000007;
	pub const NOT_PUBLIC: TypeAttributes = 0x00000000;
	pub const PUBLIC: TypeAttributes = 0x00000001;
	pub const NESTED_PUBLIC: TypeAttributes = 0x00000002;
	pub const NESTED_PRIVATE: TypeAttributes = 0x00000003;
	pub const NESTED_FAMILY: TypeAttributes = 0x00000004;
	pub const NESTED_ASSEMBLY: TypeAttributes = 0x00000005;
	pub const NESTED_FAM_AND_ASSEM: TypeAttributes = 0x00000006;
	pub const NESTED_FAM_OR_ASSEM: TypeAttributes = 0x00000007;

	//Class layout attributes
	pub const LAYOUT_MASK: TypeAttributes = 0x00000018;
	pub const AUTO_LAYOUT: TypeAttributes = 0x00000000;
	pub const SEQUENTIAL_LAYOUT: TypeAttributes = 0x00000008;
	pub const EXPLICIT_LAYOUT: TypeAttributes = 0x000000010;

	//Class semantics attributes
	pub const CLASS_SEMANTICS_MASK: TypeAttributes = 0x000000020;
	pub const SPECIAL_CLASS_SEMANTICS_MASK: TypeAttributes = 0x000000580;
	pub const CLASS: TypeAttributes = 0x000000000;
	pub const INTERFACE: TypeAttributes = 0x000000020;
	pub const ABSTRACT: TypeAttributes = 0x000000080;
	pub const SEALED: TypeAttributes = 0x0000000100;
	pub const SPECIAL_NAME: TypeAttributes = 0x000000400;

	//Implementation Attributes
	pub const IMPORT: TypeAttributes = 0x000001000;
	pub const SERIALIZABLE: TypeAttributes = 0x000002000;

	//String formatting Attributes
	pub const STRING_FORMAT_MASK: TypeAttributes = 0x0000030000;
	pub const CUSTOM_STRING_FORMAT_MASK: TypeAttributes = 0x0000C00000;
	pub const ANSI_CLASS: TypeAttributes = 0x0000000000;
	pub const UNICODE_CLASS: TypeAttributes = 0x0000010000;
	pub const AUTO_CLASS: TypeAttributes = 0x0000020000;
	pub const CUSTOM_FORMAT_CLASS: TypeAttributes = 0x0000030000;

	//Class Initialization Attributes
	pub const BEFORE_FIELD_INIT: TypeAttributes = 0x0010000000;

	//Additional Flags
	pub const RT_SPECIAL_NAME: TypeAttributes = 0x0000000800;
	pub const HAS_SECURITY: TypeAttributes = 0x0000040000;
	pub const IS_TYPE_FORWARDER: TypeAttributes = 0x0000200000;
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
		string_idx_size: MetadataIndexSize,
	) -> Result<Self, ParsingError> {
		let mut reader = ZeroCopyReader::new(bytes);
		Ok(Self {
			hash_algorithm: *reader.read::<AssemblyHashAlgorithm>()?,
			major_version: *reader.read::<u16>()?,
			minor_version: *reader.read::<u16>()?,
			build_number: *reader.read::<u16>()?,
			revision_number: *reader.read::<u16>()?,
			flags: *reader.read::<AssemblyFlags>()?,
			public_key: reader.read_index(blob_idx_size)?,
			name: reader.read_index(string_idx_size)?,
			culture: reader.read_index(string_idx_size)?,
		})
	}
}

impl MetadataTable<'_> for AssemblyTable {
	fn kind(&self) -> TableKind {
		TableKind::Assembly
	}

	fn static_kind() -> TableKind
	where
		Self: Sized,
	{
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
