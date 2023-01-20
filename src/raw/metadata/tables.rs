pub use method_impl_flags::MethodImplFlags;
pub use type_attributes::TypeAttributes;
pub use assembly_flags::AssemblyFlags;
pub use method_flags::MethodFlags;
use crate::__impl_multi_row_table;
pub use field_flags::FieldFlags;
pub use param_flags::ParamFlags;
use private::ParseRow;
use strum::EnumIter;
use crate::raw::*;
use paste::paste;

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

pub trait MetadataTable<'l>
where
	Self: MetadataTableImpl<'l> + ParseRow,
{
	type Iter: Iterator<Item = Result<Self::Row, Error>>;

	fn bytes(&self) -> &'l [u8];
	fn row_size(&self) -> usize;
	fn iter(&self) -> Self::Iter;

	fn get(&self, index: MetadataIndex) -> Result<Self::Row, Error> {
		let mut reader = ByteStream::new(self.bytes());
		reader.seek(self.row_size() * index.0)?;
		self.parse_row(&mut reader)
	}
}

__impl_multi_row_table! {
	Module

	Row(self, reader) {
		generation: u16 = reader.read()?,
		name: MetadataIndex = reader.read_index(self.str_size)?,
		module_version_id: MetadataIndex = reader.read_index(self.guid_size)?,
		enc_id: MetadataIndex = reader.read_index(self.guid_size)?,
		enc_base_id: MetadataIndex = reader.read_index(self.guid_size)?
	}

	Table(tables) {
		row_size = {
			let g = GuidHeap::idx_size(tables) as usize;
			let s = StringHeap::idx_size(tables) as usize;
			2 + s + g * 3
		}

		str_size: MetadataIndexSize = StringHeap::idx_size(tables),
		guid_size: MetadataIndexSize = GuidHeap::idx_size(tables)
	}
}

__impl_multi_row_table! {
	TypeRef

	Row(self, reader) {
		resolution_scope: MetadataIndex = reader.read_index(self.res_size)?,
		type_name: MetadataIndex = reader.read_index(self.str_size)?,
		type_namespace: MetadataIndex = reader.read_index(self.str_size)?
	}

	Table(tables) {
		row_size = {
			let s = StringHeap::idx_size(tables) as usize;
			let r = get_coded_index_size(CodedIndexKind::TypeOrMethodDef, tables) as usize;
			r + s * 2
		}

		res_size: MetadataIndexSize = StringHeap::idx_size(tables),
		str_size: MetadataIndexSize = get_coded_index_size(CodedIndexKind::TypeOrMethodDef, tables)
	}
}

__impl_multi_row_table! {
	TypeDef

	Row(self, reader) {
		flags: TypeAttributes = reader.read()?,
		name: MetadataIndex = reader.read_index(self.str_size)?,
		namespace: MetadataIndex = reader.read_index(self.str_size)?,
		extends: MetadataIndex = reader.read_index(self.ext_size)?,
		fields: MetadataIndex = reader.read_index(self.fld_size)?,
		methods: MetadataIndex = reader.read_index(self.mtd_size)?
	}

	Table(tables) {
		row_size = {
			let s = StringHeap::idx_size(tables) as usize;
			let f = tables.table_idx_size(TableKind::Field) as usize;
			let m = tables.table_idx_size(TableKind::MethodDef) as usize;
			let e = get_coded_index_size(CodedIndexKind::TypeDefOrRef, tables) as usize;
			4 + s * 2 + e + f + m
		}

		str_size: MetadataIndexSize = StringHeap::idx_size(tables),
		fld_size: MetadataIndexSize = tables.table_idx_size(TableKind::Field),
		mtd_size: MetadataIndexSize = tables.table_idx_size(TableKind::MethodDef),
		ext_size: MetadataIndexSize = get_coded_index_size(CodedIndexKind::TypeDefOrRef, tables)
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

__impl_multi_row_table! {
	Field

	Row(self, reader) {
		flags: FieldFlags = reader.read()?,
		name: MetadataIndex = reader.read_index(self.str_size)?,
		signature: MetadataIndex = reader.read_index(self.blob_size)?
	}

	Table(tables) {
		row_size = {
			let b = BlobHeap::idx_size(tables) as usize;
			let s = StringHeap::idx_size(tables) as usize;
			2 + s + b
		}

		blob_size: MetadataIndexSize = BlobHeap::idx_size(tables),
		str_size: MetadataIndexSize = StringHeap::idx_size(tables)
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

__impl_multi_row_table! {
	MethodDef

	Row(self, reader) {
		rva: u32 = reader.read()?,
		impl_flags: MethodImplFlags = reader.read()?,
		flags: MethodFlags = reader.read()?,
		name: MetadataIndex = reader.read_index(self.str_size)?,
		signature: MetadataIndex = reader.read_index(self.blob_size)?,
		params: MetadataIndex = reader.read_index(self.param_size)?
	}

	Table(tables) {
		row_size = {
			let b = BlobHeap::idx_size(tables) as usize;
			let s = StringHeap::idx_size(tables) as usize;
			let p = tables.table_idx_size(TableKind::Param) as usize;
			8 + s + b + p
		}

		str_size: MetadataIndexSize = StringHeap::idx_size(tables),
		blob_size: MetadataIndexSize = BlobHeap::idx_size(tables),
		param_size: MetadataIndexSize = tables.table_idx_size(TableKind::Param)
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

__impl_multi_row_table! {
	Param

	Row(self, reader) {
		flags: ParamFlags = reader.read()?,
		sequence: u16 = reader.read()?,
		name: MetadataIndex = reader.read_index(self.str_size)?
	}

	Table(tables) {
		row_size = {
			StringHeap::idx_size(tables) as usize + 4
		}

		str_size: MetadataIndexSize = StringHeap::idx_size(tables)
	}
}

pub mod param_flags {
	pub type ParamFlags = u16;
	pub const IN: ParamFlags = 0x0001;
	pub const OUT: ParamFlags = 0x0002;
	pub const OPTIONAL: ParamFlags = 0x0010;
	pub const HAS_DEFAULT: ParamFlags = 0x1000;
	pub const HAS_FIELD_MARSHAL: ParamFlags = 0x2000;
	pub const UNUSED: ParamFlags = 0xcfe0;
}

__impl_multi_row_table! {
	InterfaceImpl

	Row(self, reader) {
		type_: MetadataIndex = reader.read_index(self.type_size)?,
		interface: MetadataIndex = reader.read_index(self.interface_size)?
	}

	Table(tables) {
		row_size = {
			let t = tables.table_idx_size(TableKind::TypeRef) as usize;
			let i = get_coded_index_size(CodedIndexKind::TypeDefOrRef, tables) as usize;
			t + i
		}

		type_size: MetadataIndexSize = tables.table_idx_size(TableKind::TypeRef),
		interface_size: MetadataIndexSize = get_coded_index_size(CodedIndexKind::TypeDefOrRef, tables)
	}
}

__impl_multi_row_table! {
	MemberRef

	Row(self, reader) {
		parent: MetadataIndex = reader.read_index(self.parent_size)?,
		name: MetadataIndex = reader.read_index(self.str_size)?,
		signature: MetadataIndex = reader.read_index(self.blob_size)?
	}

	Table(tables) {
		row_size = {
			let b = BlobHeap::idx_size(tables) as usize;
			let s = StringHeap::idx_size(tables) as usize;
			let p = get_coded_index_size(CodedIndexKind::MemberRefParent, tables) as usize;
			p + s + b
		}

		str_size: MetadataIndexSize = BlobHeap::idx_size(tables),
		blob_size: MetadataIndexSize = StringHeap::idx_size(tables),
		parent_size: MetadataIndexSize = get_coded_index_size(CodedIndexKind::MemberRefParent, tables)
	}
}

__impl_multi_row_table! {
	CustomAttribute

	Row(self, reader) {
		parent: MetadataIndex = reader.read_index(self.parent_size)?,
		type_: MetadataIndex = reader.read_index(self.type_size)?,
		value: MetadataIndex = reader.read_index(self.blob_size)?
	}

	Table(tables) {
		row_size = {
			let b = BlobHeap::idx_size(tables) as usize;
			let p = get_coded_index_size(CodedIndexKind::HasCustomAttribute, tables) as usize;
			let t = get_coded_index_size(CodedIndexKind::CustomAttributeType, tables) as usize;
			p + t + b
		}

		blob_size: MetadataIndexSize = BlobHeap::idx_size(tables),
		type_size: MetadataIndexSize = get_coded_index_size(CodedIndexKind::CustomAttributeType, tables),
		parent_size: MetadataIndexSize = get_coded_index_size(CodedIndexKind::HasCustomAttribute, tables)
	}
}

//<editor-fold desc="Constant">
#[derive(Debug, Clone)]
pub struct Constant {
	type_: ElementType,
	parent: MetadataIndex,
	value: MetadataIndex,
}

#[derive(Clone)]
pub struct ConstantTable<'l> {
	bytes: &'l [u8],
	row_size: usize,
	blob_size: MetadataIndexSize,
	parent_size: MetadataIndexSize,
}

#[derive(Clone)]
pub struct ConstantIterator<'l> {
	reader: ByteStream<'l>,
	table: ConstantTable<'l>,
}

impl <'l> MetadataTable<'l> for ConstantTable<'l> {
	type Iter = ConstantIterator<'l>;

	fn bytes(&self) -> &'l [u8] {
		self.bytes
	}

	fn row_size(&self) -> usize {
		self.row_size
	}

	fn iter(&self) -> Self::Iter {
		ConstantIterator {
			table: self.clone(),
			reader: ByteStream::new(self.bytes),
		}
	}
}

impl ParseRow for ConstantTable<'_> {
	type Row = Constant;

	fn parse_row(&self, reader: &mut ByteStream) -> Result<Self::Row, Error> {
		let type_ = reader.read()?;
		reader.skip(1)?;
		Ok(Self::Row {
			type_,
			parent: reader.read_index(self.parent_size)?,
			value: reader.read_index(self.blob_size)?,
		})
	}
}

impl <'l> MetadataTableImpl<'l> for ConstantTable<'l> {
	fn cli_identifier() -> TableKind {
		TableKind::Constant
	}

	fn calc_row_size(tables: &TableHeap) -> usize {
		let b = BlobHeap::idx_size(tables) as usize;
		let p = get_coded_index_size(CodedIndexKind::HasConstant, tables) as usize;
		2 + p + b
	}

	fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error> {
		Ok(Self {
			bytes,
			row_size: Self::calc_row_size(tables),
			blob_size: BlobHeap::idx_size(tables),
			parent_size: get_coded_index_size(CodedIndexKind::HasConstant, tables),
		})
	}
}

impl Iterator for ConstantIterator<'_> {
	type Item = Result<Constant, Error>;

	fn next(&mut self) -> Option<Self::Item> {
		match self.reader.remaining() {
			0 => None,
			_ => Some(self.table.parse_row(&mut self.reader))
		}
	}
}

impl Constant {
	pub fn type_(&self) -> ElementType {
		self.type_
	}

	pub fn parent(&self) -> MetadataIndex {
		self.parent
	}

	pub fn value(&self) -> MetadataIndex {
		self.value
	}
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ElementType {
	End = 0x00,
	Void = 0x01,
	Bool = 0x02,
	Char = 0x03,
	I1 = 0x04,
	U1 = 0x05,
	I2 = 0x06,
	U2 = 0x07,
	I4 = 0x08,
	U4 = 0x09,
	I8 = 0x0A,
	U8 = 0x0B,
	R4 = 0x0C,
	R8 = 0x0D,
	String = 0x0E,
	Ptr = 0x0F,
	ByRef = 0x10,
	ValueType = 0x11,
	Class = 0x12,
	Var = 0x13,
	Array = 0x14,
	GenericInst = 0x15,
	TypedByRef = 0x16,
	IPtr = 0x17,
	UPtr = 0x18,
	FnPtr = 0x1B,
	Object = 0x1C,
	SzArray = 0x1D,
	MVar = 0x1E,
	CModReqd = 0x1F,
	CModOpt = 0x20,
	Internal = 0x21,
	Modifier = 0x40,
	Sentinel = 0x41,
	Pinned = 0x45,
	Type = 0x50,
}
//</editor-fold>

//<editor-fold desc="Assembly">
#[derive(Clone)]
pub struct AssemblyTable<'l> {
	bytes: &'l [u8],
	row_size: usize,
	str_size: MetadataIndexSize,
	blob_size: MetadataIndexSize,
}

#[derive(Debug, Clone)]
pub struct Assembly {
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

impl<'l> MetadataTable<'l> for AssemblyTable<'l> {
	type Iter = std::option::IntoIter<Result<Assembly, Error>>;

	fn bytes(&self) -> &'l [u8] {
		self.bytes
	}

	fn row_size(&self) -> usize {
		self.row_size
	}

	fn iter(&self) -> Self::Iter {
		let mut reader = ByteStream::new(self.bytes);
		Some(self.parse_row(&mut reader)).into_iter()
	}
}

impl ParseRow for AssemblyTable<'_> {
	type Row = Assembly;

	fn parse_row(&self, reader: &mut ByteStream) -> Result<Self::Row, Error> {
		Ok(Assembly {
			hash_algorithm: reader.read()?,
			major_version: reader.read()?,
			minor_version: reader.read()?,
			build_number: reader.read()?,
			revision_number: reader.read()?,
			flags: reader.read()?,
			public_key: reader.read_index(self.blob_size)?,
			name: reader.read_index(self.str_size)?,
			culture: reader.read_index(self.str_size)?,
		})
	}
}

impl<'l> MetadataTableImpl<'l> for AssemblyTable<'l> {
	fn cli_identifier() -> TableKind {
		TableKind::Assembly
	}

	fn calc_row_size(tables: &TableHeap) -> usize {
		let b = BlobHeap::idx_size(tables) as usize;
		let s = StringHeap::idx_size(tables) as usize;
		16 + b + s * 2
	}

	fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error> {
		Ok(Self {
			bytes,
			row_size: Self::calc_row_size(tables),
			blob_size: BlobHeap::idx_size(tables),
			str_size: StringHeap::idx_size(tables),
		})
	}
}

impl Assembly {
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
//</editor-fold>

#[derive(Clone)]
pub struct StandAloneSignatureTable<'l> {
	bytes: &'l [u8],
	blob_size: MetadataIndexSize,
}

impl<'l> MetadataTable<'l> for StandAloneSignatureTable<'l> {
	type Iter = std::option::IntoIter<Result<MetadataIndex, Error>>;

	fn bytes(&self) -> &'l [u8] {
		self.bytes
	}

	fn row_size(&self) -> usize {
		self.blob_size as usize
	}

	fn iter(&self) -> Self::Iter {
		let mut reader = ByteStream::new(self.bytes);
		Some(self.parse_row(&mut reader)).into_iter()
	}
}

impl ParseRow for StandAloneSignatureTable<'_> {
	type Row = MetadataIndex;

	fn parse_row(&self, reader: &mut ByteStream) -> Result<Self::Row, Error> {
		reader.read_index(self.blob_size)
	}
}

impl<'l> MetadataTableImpl<'l> for StandAloneSignatureTable<'l> {
	fn cli_identifier() -> TableKind {
		TableKind::StandAloneSig
	}

	fn calc_row_size(tables: &TableHeap) -> usize {
		BlobHeap::idx_size(tables) as usize
	}

	fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error> {
		Ok(Self {
			bytes,
			blob_size: BlobHeap::idx_size(tables),
		})
	}
}

pub(crate) mod private {
	use crate::raw::*;

	pub trait ParseRow {
		type Row;
		fn parse_row(&self, reader: &mut ByteStream) -> Result<Self::Row, Error>;
	}

	pub trait MetadataTableImpl<'l>
	where
		Self: Sized,
	{
		fn cli_identifier() -> TableKind;
		fn calc_row_size(tables: &TableHeap) -> usize;
		fn new(bytes: &'l [u8], tables: &TableHeap) -> Result<Self, Error>;
	}
}
