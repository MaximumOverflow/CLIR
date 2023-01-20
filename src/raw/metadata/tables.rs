pub use method_impl_flags::MethodImplFlags;
pub use type_attributes::TypeAttributes;
pub use assembly_flags::AssemblyFlags;
pub use method_flags::MethodFlags;
pub use field_flags::FieldFlags;
use strum::EnumIter;
use crate::raw::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumIter)]
pub enum TableKind {
	Module = 0x00,
	TypeRef = 0x01,
	TypeDef = 0x02,
	FieldPtr = 0x03,
	Field = 0x04,
	MethodPtr = 0x05,
	MethodDef = 0x06,
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

// #####################
// ### Module Table ###
// #####################

pub struct ModuleTable<'l> {
	bytes: &'l [u8],
	guid_size: MetadataIndexSize,
	str_size: MetadataIndexSize,
}

#[derive(Debug)]
pub struct Module {
	generation: u16,
	name: MetadataIndex,
	module_version_id: MetadataIndex,
	enc_id: MetadataIndex,
	enc_base_id: MetadataIndex,
}

pub struct ModuleIterator<'l> {
	reader: ByteStream<'l>,
	guid_size: MetadataIndexSize,
	str_size: MetadataIndexSize,
}

impl<'l> MetadataTable<'l> for ModuleTable<'l> {
	fn cli_identifier() -> TableKind {
		TableKind::Module
	}

	fn row_size(tables: &TableHeap) -> usize {
		let g = GuidHeap::idx_size(tables) as usize;
		let s = StringHeap::idx_size(tables) as usize;
		2 + s + g * 3
	}

	fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error> {
		Ok(Self {
			bytes,
			guid_size: GuidHeap::idx_size(tables),
			str_size: StringHeap::idx_size(tables),
		})
	}
}

impl ModuleTable<'_> {
	pub fn iter(&self) -> ModuleIterator {
		ModuleIterator {
			reader: ByteStream::new(self.bytes),
			guid_size: self.guid_size,
			str_size: self.str_size,
		}
	}
}

impl Module {
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

impl Iterator for ModuleIterator<'_> {
	type Item = Result<Module, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		fn parse(this: &mut ModuleIterator) -> Result<Module, Error> {
			Ok(Module {
				generation: this.reader.read::<u16>()?,
				name: this.reader.read_index(this.str_size)?,
				module_version_id: this.reader.read_index(this.guid_size)?,
				enc_id: this.reader.read_index(this.guid_size)?,
				enc_base_id: this.reader.read_index(this.guid_size)?,
			})
		}

		match self.reader.remaining() {
			0 => None,
			_ => Some(parse(self)),
		}
	}
}

// #####################
// ### TypeRef Table ###
// #####################

pub struct TypeRefTable<'l> {
	bytes: &'l [u8],
	res_size: MetadataIndexSize,
	str_size: MetadataIndexSize,
}

#[derive(Debug, Clone)]
pub struct TypeRef {
	resolution_scope: MetadataIndex,
	type_name: MetadataIndex,
	type_namespace: MetadataIndex,
}

pub struct TypeRefIterator<'l> {
	reader: ByteStream<'l>,
	res_size: MetadataIndexSize,
	str_size: MetadataIndexSize,
}

impl<'l> MetadataTable<'l> for TypeRefTable<'l> {
	fn cli_identifier() -> TableKind {
		TableKind::TypeRef
	}

	fn row_size(tables: &TableHeap) -> usize {
		let s = StringHeap::idx_size(tables) as usize;
		let r = get_coded_index_size(CodedIndexKind::TypeOrMethodDef, tables) as usize;
		r + s * 2
	}

	fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error> {
		Ok(Self {
			bytes,
			res_size: StringHeap::idx_size(tables),
			str_size: get_coded_index_size(CodedIndexKind::TypeOrMethodDef, tables),
		})
	}
}

impl TypeRefTable<'_> {
	pub fn iter(&self) -> TypeRefIterator {
		TypeRefIterator {
			reader: ByteStream::new(self.bytes),
			res_size: self.res_size,
			str_size: self.str_size,
		}
	}
}

impl TypeRef {
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

impl Iterator for TypeRefIterator<'_> {
	type Item = Result<TypeRef, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		fn parse(this: &mut TypeRefIterator) -> Result<TypeRef, Error> {
			Ok(TypeRef {
				resolution_scope: this.reader.read_index(this.res_size)?,
				type_name: this.reader.read_index(this.str_size)?,
				type_namespace: this.reader.read_index(this.str_size)?,
			})
		}

		match self.reader.remaining() {
			0 => None,
			_ => Some(parse(self)),
		}
	}
}

// #####################
// ### TypeDef Table ###
// #####################

pub struct TypeDefTable<'l> {
	bytes: &'l [u8],
	str_size: MetadataIndexSize,
	ext_size: MetadataIndexSize,
	fld_size: MetadataIndexSize,
	mtd_size: MetadataIndexSize,
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

pub struct TypeDefIterator<'l> {
	reader: ByteStream<'l>,
	str_size: MetadataIndexSize,
	ext_size: MetadataIndexSize,
	fld_size: MetadataIndexSize,
	mtd_size: MetadataIndexSize,
}

impl<'l> MetadataTable<'l> for TypeDefTable<'l> {
	fn cli_identifier() -> TableKind {
		TableKind::TypeDef
	}

	fn row_size(tables: &TableHeap) -> usize {
		let s = StringHeap::idx_size(tables) as usize;
		let f = tables.table_idx_size(TableKind::Field) as usize;
		let m = tables.table_idx_size(TableKind::MethodDef) as usize;
		let e = get_coded_index_size(CodedIndexKind::TypeDefOrRef, tables) as usize;
		4 + s * 2 + e + f + m
	}

	fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error> {
		Ok(Self {
			bytes,
			str_size: StringHeap::idx_size(tables),
			fld_size: tables.table_idx_size(TableKind::Field),
			mtd_size: tables.table_idx_size(TableKind::MethodDef),
			ext_size: get_coded_index_size(CodedIndexKind::TypeDefOrRef, tables),
		})
	}
}

impl TypeDefTable<'_> {
	pub fn iter(&self) -> TypeDefIterator {
		TypeDefIterator {
			reader: ByteStream::new(self.bytes),
			str_size: self.str_size,
			ext_size: self.ext_size,
			fld_size: self.fld_size,
			mtd_size: self.mtd_size,
		}
	}
}

impl TypeDef {
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

impl Iterator for TypeDefIterator<'_> {
	type Item = Result<TypeDef, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		fn parse(this: &mut TypeDefIterator) -> Result<TypeDef, Error> {
			Ok(TypeDef {
				flags: this.reader.read::<TypeAttributes>()?,
				name: this.reader.read_index(this.str_size)?,
				namespace: this.reader.read_index(this.str_size)?,
				extends: this.reader.read_index(this.ext_size)?,
				fields: this.reader.read_index(this.fld_size)?,
				methods: this.reader.read_index(this.mtd_size)?,
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
	pub const NESTED_FAMILY_AND_ASSEMBLY: TypeAttributes = 0x00000006;
	pub const NESTED_FAMILY_OR_ASSEMBLY: TypeAttributes = 0x00000007;

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

// #######################
// ### MethodDef Table ###
// #######################

pub struct MethodDefTable<'l> {
	bytes: &'l [u8],
	str_size: MetadataIndexSize,
	blob_size: MetadataIndexSize,
	param_size: MetadataIndexSize,
}

#[derive(Debug, Clone)]
pub struct MethodDef {
	rva: u32,
	impl_flags: MethodImplFlags,
	flags: MethodFlags,
	name: MetadataIndex,
	signature: MetadataIndex,
	params: MetadataIndex,
}

pub struct MethodDefIterator<'l> {
	reader: ByteStream<'l>,
	str_size: MetadataIndexSize,
	blob_size: MetadataIndexSize,
	param_size: MetadataIndexSize,
}

impl <'l> MetadataTable<'l> for MethodDefTable<'l> {
	fn cli_identifier() -> TableKind {
		TableKind::MethodDef
	}

	fn row_size(tables: &TableHeap) -> usize {
		let b = BlobHeap::idx_size(tables) as usize;
		let s = StringHeap::idx_size(tables) as usize;
		let p = tables.table_idx_size(TableKind::Param) as usize;
		8 + s + b + p
	}

	fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error> {
		Ok(Self {
			bytes,
			str_size: StringHeap::idx_size(tables),
			blob_size: BlobHeap::idx_size(tables),
			param_size: tables.table_idx_size(TableKind::Param),
		})
	}
}

impl MethodDefTable<'_> {
	pub fn iter(&self) -> MethodDefIterator {
		MethodDefIterator {
			reader: ByteStream::new(self.bytes),
			str_size: self.str_size,
			blob_size: self.blob_size,
			param_size: self.param_size,
		}
	}
}

impl MethodDef {
	pub fn rva(&self) -> u32 {
		self.rva
	}
	
	pub fn impl_flags(&self) -> MethodImplFlags {
		self.impl_flags
	}
	
	pub fn flags(&self) -> MethodFlags {
		self.flags
	}
	
	pub fn name(&self) -> MetadataIndex {
		self.name
	}
	
	pub fn signature(&self) -> MetadataIndex {
		self.signature
	}
	
	pub fn params(&self) -> MetadataIndex {
		self.params
	}
}

impl Iterator for MethodDefIterator<'_> {
	type Item = Result<MethodDef, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		fn parse(this: &mut MethodDefIterator) -> Result<MethodDef, Error> {
			Ok(MethodDef {
				rva: this.reader.read::<u32>()?,
				impl_flags: this.reader.read::<MethodImplFlags>()?,
				flags: this.reader.read::<MethodFlags>()?,
				name: this.reader.read_index(this.str_size)?,
				signature: this.reader.read_index(this.blob_size)?,
				params: this.reader.read_index(this.param_size)?,
			})
		}

		match self.reader.remaining() {
			0 => None,
			_ => Some(parse(self)),
		}
	}
}

pub mod method_impl_flags {
	pub type MethodImplFlags = u16;
	pub const CODE_TYPE_MASK: MethodImplFlags = 0x0003;
	pub const IL: MethodImplFlags = 0x0000;
	pub const NATIVE: MethodImplFlags = 0x0001;
	pub const OPT_IL: MethodImplFlags = 0x0002;
	pub const RUNTIME: MethodImplFlags = 0x0003;
	pub const MANAGED_MASK: MethodImplFlags = 0x0004;
	pub const UNMANAGED: MethodImplFlags = 0x0004;
	pub const MANAGED: MethodImplFlags = 0x0000;
}

pub mod method_flags {
	pub type MethodFlags = u16;
	pub const MEMBER_ACCESS_MASK: MethodFlags = 0x0007;
	pub const COMPILER_CONTROLLED: MethodFlags = 0x0000;
	pub const PRIVATE: MethodFlags = 0x0001;
	pub const FAMILY_AND_ASSEMBLY: MethodFlags = 0x0002;
	pub const ASSEMBLY: MethodFlags = 0x0003;
	pub const FAMILY: MethodFlags = 0x0004;
	pub const FAMILY_OR_ASSEMBLY: MethodFlags = 0x0005;
	pub const PUBLIC: MethodFlags = 0x0006;
	pub const STATIC: MethodFlags = 0x0010;
	pub const FINAL: MethodFlags = 0x0020;
	pub const VIRTUAL: MethodFlags = 0x0040;
	pub const HIDE_BY_SIGNATURE: MethodFlags = 0x0080;
	pub const VTABLE_LAYOUT_MASK: MethodFlags = 0x0100;
	pub const REUSE_SLOT: MethodFlags = 0x0000;
	pub const NEW_SLOT: MethodFlags = 0x0100;
	pub const STRICT: MethodFlags = 0x0200;
	pub const ABSTRACT: MethodFlags = 0x0400;
	pub const SPECIAL_NAME: MethodFlags = 0x0800;
	pub const PINVOKE_IMPL: MethodFlags = 0x2000;
	pub const UNMANAGED_EXPORT: MethodFlags = 0x0008;
	pub const RT_SPECIAL_NAME: MethodFlags = 0x1000;
	pub const HAS_SECURITY: MethodFlags = 0x4000;
	pub const REQUIRE_SECURITY_OBJECT: MethodFlags = 0x8000;
}

// ######################
// ### Assembly Table ###
// ######################

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

#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum AssemblyHashAlgorithm {
	None = 0x0000,
	MD5 = 0x8003,
	SHA1 = 0x8004,
}

impl MetadataTable<'_> for AssemblyTable {
	fn cli_identifier() -> TableKind {
		TableKind::Assembly
	}

	fn row_size(tables: &TableHeap) -> usize {
		let b = BlobHeap::idx_size(tables) as usize;
		let s = StringHeap::idx_size(tables) as usize;
		16 + b + s * 2
	}

	fn new(bytes: &'_ [u8], tables: &TableHeap) -> Result<Self, Error> {
		let mut reader = ByteStream::new(bytes);
		let blob_size = BlobHeap::idx_size(tables);
		let str_size = StringHeap::idx_size(tables);

		Ok(Self {
			hash_algorithm: reader.read::<AssemblyHashAlgorithm>()?,
			major_version: reader.read::<u16>()?,
			minor_version: reader.read::<u16>()?,
			build_number: reader.read::<u16>()?,
			revision_number: reader.read::<u16>()?,
			flags: reader.read::<AssemblyFlags>()?,
			public_key: reader.read_index(blob_size)?,
			name: reader.read_index(str_size)?,
			culture: reader.read_index(str_size)?,
		})
	}
}

impl AssemblyTable {
	pub fn hash_algorithm(&self) -> AssemblyHashAlgorithm {
		self.hash_algorithm
	}
	pub fn major_version(&self) -> u16 {
		self.major_version
	}
	pub fn minor_version(&self) -> u16 {
		self.minor_version
	}
	pub fn build_number(&self) -> u16 {
		self.build_number
	}
	pub fn revision_number(&self) -> u16 {
		self.revision_number
	}
	pub fn flags(&self) -> AssemblyFlags {
		self.flags
	}
	pub fn public_key(&self) -> MetadataIndex {
		self.public_key
	}
	pub fn name(&self) -> MetadataIndex {
		self.name
	}
	pub fn culture(&self) -> MetadataIndex {
		self.culture
	}
}

pub mod assembly_flags {
	pub type AssemblyFlags = u32;
	pub const PUBLIC_KEY: AssemblyFlags = 0x0001;
	pub const RETARGETABLE: AssemblyFlags = 0x0100;
	pub const DISABLE_JIT_COMPILE_OPTIMIZER: AssemblyFlags = 0x4000;
	pub const ENABLE_JIT_COMPILE_TRACKING: AssemblyFlags = 0x8000;
}

// #####################
// ### TypeDef Table ###
// #####################

pub struct FieldTable<'l> {
	bytes: &'l [u8],
	str_size: MetadataIndexSize,
	blob_size: MetadataIndexSize,
}

pub struct Field {
	flags: FieldFlags,
	name: MetadataIndex,
	signature: MetadataIndex,
}

pub struct FieldIterator<'l> {
	reader: ByteStream<'l>,
	str_size: MetadataIndexSize,
	blob_size: MetadataIndexSize,
}

impl<'l> MetadataTable<'l> for FieldTable<'l> {
	fn cli_identifier() -> TableKind {
		TableKind::Field
	}

	fn row_size(tables: &TableHeap) -> usize {
		let b = BlobHeap::idx_size(tables) as usize;
		let s = StringHeap::idx_size(tables) as usize;
		2 + s + b
	}

	fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error> {
		Ok(Self {
			bytes,
			blob_size: BlobHeap::idx_size(tables),
			str_size: StringHeap::idx_size(tables),
		})
	}
}

impl FieldTable<'_> {
	pub fn iter(&self) -> FieldIterator {
		FieldIterator {
			reader: ByteStream::new(self.bytes),
			str_size: self.str_size,
			blob_size: self.blob_size,
		}
	}
}

impl Field {
	pub fn flags(&self) -> FieldFlags {
		self.flags
	}

	pub fn name(&self) -> MetadataIndex {
		self.name
	}

	pub fn signature(&self) -> MetadataIndex {
		self.signature
	}
}

impl Iterator for FieldIterator<'_> {
	type Item = Result<Field, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		fn parse(this: &mut FieldIterator) -> Result<Field, Error> {
			Ok(Field {
				flags: this.reader.read::<FieldFlags>()?,
				name: this.reader.read_index(this.str_size)?,
				signature: this.reader.read_index(this.blob_size)?,
			})
		}

		match self.reader.remaining() {
			0 => None,
			_ => Some(parse(self)),
		}
	}
}

pub mod field_flags {
	pub type FieldFlags = u16;
	pub const FIELD_ACCESS_MASK: FieldFlags = 0x0007;
	pub const COMPILER_CONTROLLED: FieldFlags = 0x0000;
	pub const PRIVATE: FieldFlags = 0x0001;
	pub const FAMILY_AND_ASSEMBLY: FieldFlags = 0x0002;
	pub const ASSEMBLY: FieldFlags = 0x0003;
	pub const FAMILY: FieldFlags = 0x0004;
	pub const FAMILY_OR_ASSEMBLY: FieldFlags = 0x0005;
	pub const PUBLIC: FieldFlags = 0x0006;
	pub const STATIC: FieldFlags = 0x0010;
	pub const INIT_ONLY: FieldFlags = 0x0020;
	pub const LITERAL: FieldFlags = 0x0040;
	pub const NOT_SERIALIZED: FieldFlags = 0x0080;
	pub const SPECIAL_NAME: FieldFlags = 0x0200;
	pub const PINVOKE_IMPL: FieldFlags = 0x2000;
	pub const RT_SPECIAL_NAME: FieldFlags = 0x0400;
	pub const HAS_FIELD_MARSHAL: FieldFlags = 0x1000;
	pub const HAS_DEFAULT: FieldFlags = 0x8000;
	pub const HAS_FIELD_RVA: FieldFlags = 0x0100;	
}

pub(crate) mod private {
	use crate::raw::*;

	pub trait MetadataTable<'l>
	where
		Self: Sized,
	{
		fn cli_identifier() -> TableKind;
		fn row_size(tables: &TableHeap) -> usize;
		fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error>;

		fn kind(&self) -> TableKind {
			Self::cli_identifier()
		}
	}
}
