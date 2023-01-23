use std::fmt::{Debug, Display, Formatter};
use crate::raw::*;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct TableIndex(pub(crate) u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct HeapIndex(pub(crate) u32);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct CodedIndex(pub(crate) u32);

impl Debug for CodedIndex {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "0x{:X}", self.0)
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct MetadataToken(pub(crate) u32);

impl Display for MetadataToken {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "0x{:X} as {:?}", self.index(), self.token_kind())
	}
}

impl Debug for MetadataToken {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "0x{:X}", self.0)
	}
}

#[derive(Debug, Copy, Clone)]
pub enum IndexSize {
	Slim = 0x2,
	Fat = 0x4,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CodedIndexKind {
	TypeDefOrRef,
	HasConstant,
	HasCustomAttribute,
	HasFieldMarshal,
	HasDeclSecurity,
	MemberRefParent,
	HasSemantics,
	MethodDefOrRef,
	MemberForwarded,
	Implementation,
	CustomAttributeType,
	ResolutionScope,
	TypeOrMethodDef,
	HasCustomDebugInformation,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MetadataTokenKind {
	Module = 0x00000000,
	TypeRef = 0x01000000,
	TypeDef = 0x02000000,
	Field = 0x04000000,
	Method = 0x06000000,
	Param = 0x08000000,
	InterfaceImpl = 0x09000000,
	MemberRef = 0x0a000000,
	CustomAttribute = 0x0c000000,
	Permission = 0x0e000000,
	Signature = 0x11000000,
	Event = 0x14000000,
	Property = 0x17000000,
	ModuleRef = 0x1a000000,
	TypeSpec = 0x1b000000,
	Assembly = 0x20000000,
	AssemblyRef = 0x23000000,
	File = 0x26000000,
	ExportedType = 0x27000000,
	ManifestResource = 0x28000000,
	GenericParam = 0x2a000000,
	MethodSpec = 0x2b000000,
	GenericParamConstraint = 0x2c000000,

	Document = 0x30000000,
	MethodDebugInformation = 0x31000000,
	LocalScope = 0x32000000,
	LocalVariable = 0x33000000,
	LocalConstant = 0x34000000,
	ImportScope = 0x35000000,
	StateMachineMethod = 0x36000000,
	CustomDebugInformation = 0x37000000,

	String = 0x70000000,
}

impl MetadataToken {
	pub(crate) fn new(index: u32, kind: MetadataTokenKind) -> MetadataToken {
		if index == 0 {
			MetadataToken(0)
		} else {
			MetadataToken(index | kind as u32)
		}
	}

	pub fn is_null(&self) -> bool {
		self.0 == 0
	}

	pub fn index(&self) -> usize {
		(self.0 & 0x00FFFFFF) as usize
	}

	pub fn token_kind(&self) -> MetadataTokenKind {
		unsafe { std::mem::transmute(self.0 & 0xFF000000) }
	}
}

impl CodedIndex {
	pub fn get_size(kind: CodedIndexKind, tables_heap: &TableHeap) -> IndexSize {
		let (bits, tables): (usize, &[TableKind]) = match kind {
			CodedIndexKind::TypeDefOrRef => (2, &[TableKind::TypeDef, TableKind::TypeRef, TableKind::TypeSpec]),
			CodedIndexKind::HasConstant => (2, &[TableKind::Field, TableKind::Param, TableKind::Property]),
			CodedIndexKind::HasCustomAttribute => (
				5,
				&[
					TableKind::MethodDef,
					TableKind::Field,
					TableKind::TypeRef,
					TableKind::TypeDef,
					TableKind::Param,
					TableKind::InterfaceImpl,
					TableKind::MemberRef,
					TableKind::Module,
					TableKind::DeclSecurity,
					TableKind::Property,
					TableKind::Event,
					TableKind::StandAloneSig,
					TableKind::ModuleRef,
					TableKind::TypeSpec,
					TableKind::Assembly,
					TableKind::AssemblyRef,
					TableKind::File,
					TableKind::ExportedType,
					TableKind::ManifestResource,
					TableKind::GenericParam,
					TableKind::GenericParamConstraint,
					TableKind::MethodSpec,
				],
			),
			CodedIndexKind::HasFieldMarshal => (1, &[TableKind::Field, TableKind::Param]),
			CodedIndexKind::HasDeclSecurity => (2, &[TableKind::TypeDef, TableKind::MethodDef, TableKind::Assembly]),
			CodedIndexKind::MemberRefParent => (
				3,
				&[
					TableKind::TypeDef,
					TableKind::TypeRef,
					TableKind::ModuleRef,
					TableKind::MethodDef,
					TableKind::TypeSpec,
				],
			),
			CodedIndexKind::HasSemantics => (1, &[TableKind::Event, TableKind::Property]),
			CodedIndexKind::MethodDefOrRef => (1, &[TableKind::MethodDef, TableKind::MemberRef]),
			CodedIndexKind::MemberForwarded => (1, &[TableKind::Field, TableKind::MethodDef]),
			CodedIndexKind::Implementation => (2, &[TableKind::File, TableKind::AssemblyRef, TableKind::ExportedType]),
			CodedIndexKind::CustomAttributeType => (3, &[TableKind::MethodDef, TableKind::MemberRef]),
			CodedIndexKind::ResolutionScope => (
				2,
				&[
					TableKind::Module,
					TableKind::ModuleRef,
					TableKind::AssemblyRef,
					TableKind::TypeRef,
				],
			),
			CodedIndexKind::TypeOrMethodDef => (1, &[TableKind::TypeDef, TableKind::MethodDef]),
			CodedIndexKind::HasCustomDebugInformation => (
				5,
				&[
					TableKind::MethodDef,
					TableKind::Field,
					TableKind::TypeRef,
					TableKind::TypeDef,
					TableKind::Param,
					TableKind::InterfaceImpl,
					TableKind::MemberRef,
					TableKind::Module,
					TableKind::DeclSecurity,
					TableKind::Property,
					TableKind::Event,
					TableKind::StandAloneSig,
					TableKind::ModuleRef,
					TableKind::TypeSpec,
					TableKind::Assembly,
					TableKind::AssemblyRef,
					TableKind::File,
					TableKind::ExportedType,
					TableKind::ManifestResource,
					TableKind::GenericParam,
					TableKind::GenericParamConstraint,
					TableKind::MethodSpec,
					TableKind::Document,
					TableKind::LocalScope,
					TableKind::LocalVariable,
					TableKind::LocalConstant,
					TableKind::ImportScope,
				],
			),
		};

		let map = |t: &TableKind| tables_heap.row_count(*t);
		match tables.iter().map(map).max().unwrap() < (1 << (16 - bits)) {
			true => IndexSize::Slim,
			false => IndexSize::Fat,
		}
	}

	pub fn decode(&self, kind: CodedIndexKind) -> Option<MetadataToken> {
		let (index, kind) = match kind {
			CodedIndexKind::TypeDefOrRef => Some((
				self.0 >> 2,
				[
					MetadataTokenKind::TypeDef,
					MetadataTokenKind::TypeRef,
					MetadataTokenKind::TypeSpec,
				]
				.get((self.0 & 3) as usize)?
				.clone(),
			)),

			CodedIndexKind::HasConstant => Some((
				self.0 >> 2,
				[
					MetadataTokenKind::Field,
					MetadataTokenKind::Param,
					MetadataTokenKind::Property,
				]
				.get((self.0 & 3) as usize)?
				.clone(),
			)),

			CodedIndexKind::HasCustomAttribute => Some((
				self.0 >> 5,
				[
					MetadataTokenKind::Method,
					MetadataTokenKind::Field,
					MetadataTokenKind::TypeRef,
					MetadataTokenKind::TypeDef,
					MetadataTokenKind::Param,
					MetadataTokenKind::InterfaceImpl,
					MetadataTokenKind::MemberRef,
					MetadataTokenKind::Module,
					MetadataTokenKind::Permission,
					MetadataTokenKind::Property,
					MetadataTokenKind::Event,
					MetadataTokenKind::Signature,
					MetadataTokenKind::ModuleRef,
					MetadataTokenKind::TypeSpec,
					MetadataTokenKind::Assembly,
					MetadataTokenKind::File,
					MetadataTokenKind::ExportedType,
					MetadataTokenKind::ManifestResource,
					MetadataTokenKind::GenericParam,
					MetadataTokenKind::GenericParamConstraint,
					MetadataTokenKind::MethodSpec,
				]
				.get((self.0 & 31) as usize)?
				.clone(),
			)),

			CodedIndexKind::HasFieldMarshal => Some((
				self.0 >> 1,
				[MetadataTokenKind::Field, MetadataTokenKind::Param]
					.get(((self.0 & 1) as usize) as usize)?
					.clone(),
			)),

			CodedIndexKind::HasDeclSecurity => Some((
				self.0 >> 2,
				[
					MetadataTokenKind::TypeDef,
					MetadataTokenKind::Method,
					MetadataTokenKind::Assembly,
				]
				.get((self.0 & 3) as usize)?
				.clone(),
			)),

			CodedIndexKind::MemberRefParent => Some((
				self.0 >> 3,
				[
					MetadataTokenKind::TypeDef,
					MetadataTokenKind::TypeRef,
					MetadataTokenKind::ModuleRef,
					MetadataTokenKind::Method,
					MetadataTokenKind::TypeSpec,
				]
				.get((self.0 & 7) as usize)?
				.clone(),
			)),

			CodedIndexKind::HasSemantics => Some((
				self.0 >> 1,
				[MetadataTokenKind::Event, MetadataTokenKind::Property]
					.get((self.0 & 1) as usize)?
					.clone(),
			)),

			CodedIndexKind::MethodDefOrRef => Some((
				self.0 >> 1,
				[MetadataTokenKind::Method, MetadataTokenKind::MemberRef]
					.get((self.0 & 1) as usize)?
					.clone(),
			)),

			CodedIndexKind::MemberForwarded => Some((
				self.0 >> 1,
				[MetadataTokenKind::Field, MetadataTokenKind::Method]
					.get((self.0 & 1) as usize)?
					.clone(),
			)),

			CodedIndexKind::Implementation => Some((
				self.0 >> 2,
				[
					MetadataTokenKind::File,
					MetadataTokenKind::AssemblyRef,
					MetadataTokenKind::ExportedType,
				]
				.get((self.0 & 3) as usize)?
				.clone(),
			)),

			CodedIndexKind::CustomAttributeType => Some((
				self.0 >> 3,
				match (self.0 & 7) as usize {
					2 => Some(MetadataTokenKind::Method),
					3 => Some(MetadataTokenKind::MemberRef),
					_ => None,
				}?,
			)),

			CodedIndexKind::ResolutionScope => Some((
				self.0 >> 2,
				[
					MetadataTokenKind::Module,
					MetadataTokenKind::ModuleRef,
					MetadataTokenKind::AssemblyRef,
					MetadataTokenKind::TypeRef,
				]
				.get((self.0 & 3) as usize)?
				.clone(),
			)),

			CodedIndexKind::TypeOrMethodDef => Some((
				self.0 >> 1,
				[MetadataTokenKind::TypeDef, MetadataTokenKind::Method]
					.get((self.0 & 1) as usize)?
					.clone(),
			)),

			CodedIndexKind::HasCustomDebugInformation => Some((
				self.0 >> 5,
				[
					MetadataTokenKind::Method,
					MetadataTokenKind::Field,
					MetadataTokenKind::TypeRef,
					MetadataTokenKind::TypeDef,
					MetadataTokenKind::Param,
					MetadataTokenKind::InterfaceImpl,
					MetadataTokenKind::MemberRef,
					MetadataTokenKind::Module,
					MetadataTokenKind::Permission,
					MetadataTokenKind::Property,
					MetadataTokenKind::Event,
					MetadataTokenKind::Signature,
					MetadataTokenKind::ModuleRef,
					MetadataTokenKind::TypeSpec,
					MetadataTokenKind::Assembly,
					MetadataTokenKind::File,
					MetadataTokenKind::ExportedType,
					MetadataTokenKind::ManifestResource,
					MetadataTokenKind::GenericParam,
					MetadataTokenKind::GenericParamConstraint,
					MetadataTokenKind::MethodSpec,
					MetadataTokenKind::Document,
					MetadataTokenKind::LocalScope,
					MetadataTokenKind::LocalVariable,
					MetadataTokenKind::LocalConstant,
					MetadataTokenKind::ImportScope,
				]
				.get((self.0 & 31) as usize)?
				.clone(),
			)),
		}?;

		Some(MetadataToken::new(index, kind))
	}

	//noinspection DuplicatedCode
	pub fn encode(index: usize, token_kind: MetadataTokenKind, kind: CodedIndexKind) -> Option<Self> {
		if index == 0 {
			return Some(CodedIndex(0));
		}

		let index: u32 = index.try_into().ok()?;

		match kind {
			CodedIndexKind::TypeDefOrRef => match token_kind {
				MetadataTokenKind::TypeDef => Some(CodedIndex((index << 2) | 0x00)),
				MetadataTokenKind::TypeRef => Some(CodedIndex((index << 2) | 0x01)),
				MetadataTokenKind::TypeSpec => Some(CodedIndex((index << 2) | 0x02)),
				_ => None,
			},

			CodedIndexKind::HasConstant => match token_kind {
				MetadataTokenKind::Field => Some(CodedIndex((index << 2) | 0x00)),
				MetadataTokenKind::Param => Some(CodedIndex((index << 2) | 0x01)),
				MetadataTokenKind::Property => Some(CodedIndex((index << 2) | 0x02)),
				_ => None,
			},

			CodedIndexKind::HasCustomAttribute => match token_kind {
				MetadataTokenKind::Method => Some(CodedIndex((index << 5) | 0x00)),
				MetadataTokenKind::Field => Some(CodedIndex((index << 5) | 0x01)),
				MetadataTokenKind::TypeRef => Some(CodedIndex((index << 5) | 0x02)),
				MetadataTokenKind::TypeDef => Some(CodedIndex((index << 5) | 0x03)),
				MetadataTokenKind::Param => Some(CodedIndex((index << 5) | 0x04)),
				MetadataTokenKind::InterfaceImpl => Some(CodedIndex((index << 5) | 0x05)),
				MetadataTokenKind::MemberRef => Some(CodedIndex((index << 5) | 0x06)),
				MetadataTokenKind::Module => Some(CodedIndex((index << 5) | 0x07)),
				MetadataTokenKind::Permission => Some(CodedIndex((index << 5) | 0x08)),
				MetadataTokenKind::Property => Some(CodedIndex((index << 5) | 0x09)),
				MetadataTokenKind::Event => Some(CodedIndex((index << 5) | 0x0A)),
				MetadataTokenKind::Signature => Some(CodedIndex((index << 5) | 0x0B)),
				MetadataTokenKind::ModuleRef => Some(CodedIndex((index << 5) | 0x0C)),
				MetadataTokenKind::TypeSpec => Some(CodedIndex((index << 5) | 0x0D)),
				MetadataTokenKind::Assembly => Some(CodedIndex((index << 5) | 0x0E)),
				MetadataTokenKind::AssemblyRef => Some(CodedIndex((index << 5) | 0x0F)),
				MetadataTokenKind::File => Some(CodedIndex((index << 5) | 0x10)),
				MetadataTokenKind::ExportedType => Some(CodedIndex((index << 5) | 0x11)),
				MetadataTokenKind::ManifestResource => Some(CodedIndex((index << 5) | 0x12)),
				MetadataTokenKind::GenericParam => Some(CodedIndex((index << 5) | 0x13)),
				MetadataTokenKind::GenericParamConstraint => Some(CodedIndex((index << 5) | 0x14)),
				MetadataTokenKind::MethodSpec => Some(CodedIndex((index << 5) | 0x15)),
				_ => None,
			},

			CodedIndexKind::HasFieldMarshal => match token_kind {
				MetadataTokenKind::Field => Some(CodedIndex((index << 1) | 0x00)),
				MetadataTokenKind::Param => Some(CodedIndex((index << 1) | 0x01)),
				_ => None,
			},

			CodedIndexKind::HasDeclSecurity => match token_kind {
				MetadataTokenKind::TypeDef => Some(CodedIndex((index << 2) | 0x00)),
				MetadataTokenKind::Method => Some(CodedIndex((index << 2) | 0x01)),
				MetadataTokenKind::Assembly => Some(CodedIndex((index << 2) | 0x02)),
				_ => None,
			},

			CodedIndexKind::MemberRefParent => match token_kind {
				MetadataTokenKind::TypeDef => Some(CodedIndex((index << 3) | 0x00)),
				MetadataTokenKind::TypeRef => Some(CodedIndex((index << 3) | 0x01)),
				MetadataTokenKind::ModuleRef => Some(CodedIndex((index << 3) | 0x02)),
				MetadataTokenKind::Method => Some(CodedIndex((index << 3) | 0x03)),
				MetadataTokenKind::TypeSpec => Some(CodedIndex((index << 3) | 0x04)),
				_ => None,
			},

			CodedIndexKind::HasSemantics => match token_kind {
				MetadataTokenKind::Event => Some(CodedIndex((index << 1) | 0x00)),
				MetadataTokenKind::Property => Some(CodedIndex((index << 1) | 0x01)),
				_ => None,
			},

			CodedIndexKind::MethodDefOrRef => match token_kind {
				MetadataTokenKind::Method => Some(CodedIndex((index << 1) | 0x00)),
				MetadataTokenKind::MemberRef => Some(CodedIndex((index << 1) | 0x01)),
				_ => None,
			},

			CodedIndexKind::MemberForwarded => match token_kind {
				MetadataTokenKind::Field => Some(CodedIndex((index << 1) | 0x00)),
				MetadataTokenKind::Method => Some(CodedIndex((index << 1) | 0x01)),
				_ => None,
			},

			CodedIndexKind::Implementation => match token_kind {
				MetadataTokenKind::File => Some(CodedIndex((index << 2) | 0x00)),
				MetadataTokenKind::AssemblyRef => Some(CodedIndex((index << 2) | 0x01)),
				MetadataTokenKind::ExportedType => Some(CodedIndex((index << 2) | 0x02)),
				_ => None,
			},

			CodedIndexKind::CustomAttributeType => match token_kind {
				MetadataTokenKind::Method => Some(CodedIndex((index << 3) | 0x02)),
				MetadataTokenKind::MemberRef => Some(CodedIndex((index << 3) | 0x03)),
				_ => None,
			},

			CodedIndexKind::ResolutionScope => match token_kind {
				MetadataTokenKind::Module => Some(CodedIndex((index << 2) | 0x00)),
				MetadataTokenKind::ModuleRef => Some(CodedIndex((index << 2) | 0x01)),
				MetadataTokenKind::AssemblyRef => Some(CodedIndex((index << 2) | 0x02)),
				MetadataTokenKind::TypeRef => Some(CodedIndex((index << 2) | 0x03)),
				_ => None,
			},

			CodedIndexKind::TypeOrMethodDef => match token_kind {
				MetadataTokenKind::TypeDef => Some(CodedIndex((index << 1) | 0x00)),
				MetadataTokenKind::Method => Some(CodedIndex((index << 1) | 0x01)),
				_ => None,
			},

			CodedIndexKind::HasCustomDebugInformation => match token_kind {
				MetadataTokenKind::Method => Some(CodedIndex((index << 5) | 0x00)),
				MetadataTokenKind::Field => Some(CodedIndex((index << 5) | 0x01)),
				MetadataTokenKind::TypeRef => Some(CodedIndex((index << 5) | 0x02)),
				MetadataTokenKind::TypeDef => Some(CodedIndex((index << 5) | 0x03)),
				MetadataTokenKind::Param => Some(CodedIndex((index << 5) | 0x04)),
				MetadataTokenKind::InterfaceImpl => Some(CodedIndex((index << 5) | 0x05)),
				MetadataTokenKind::MemberRef => Some(CodedIndex((index << 5) | 0x06)),
				MetadataTokenKind::Module => Some(CodedIndex((index << 5) | 0x07)),
				MetadataTokenKind::Permission => Some(CodedIndex((index << 5) | 0x08)),
				MetadataTokenKind::Property => Some(CodedIndex((index << 5) | 0x09)),
				MetadataTokenKind::Event => Some(CodedIndex((index << 5) | 0x0A)),
				MetadataTokenKind::Signature => Some(CodedIndex((index << 5) | 0x0B)),
				MetadataTokenKind::ModuleRef => Some(CodedIndex((index << 5) | 0x0C)),
				MetadataTokenKind::TypeSpec => Some(CodedIndex((index << 5) | 0x0D)),
				MetadataTokenKind::Assembly => Some(CodedIndex((index << 5) | 0x0E)),
				MetadataTokenKind::AssemblyRef => Some(CodedIndex((index << 5) | 0x0F)),
				MetadataTokenKind::File => Some(CodedIndex((index << 5) | 0x10)),
				MetadataTokenKind::ExportedType => Some(CodedIndex((index << 5) | 0x11)),
				MetadataTokenKind::ManifestResource => Some(CodedIndex((index << 5) | 0x12)),
				MetadataTokenKind::GenericParam => Some(CodedIndex((index << 5) | 0x13)),
				MetadataTokenKind::GenericParamConstraint => Some(CodedIndex((index << 5) | 0x14)),
				MetadataTokenKind::MethodSpec => Some(CodedIndex((index << 5) | 0x15)),
				MetadataTokenKind::Document => Some(CodedIndex((index << 5) | 0x16)),
				MetadataTokenKind::LocalScope => Some(CodedIndex((index << 5) | 0x17)),
				MetadataTokenKind::LocalVariable => Some(CodedIndex((index << 5) | 0x18)),
				MetadataTokenKind::LocalConstant => Some(CodedIndex((index << 5) | 0x19)),
				MetadataTokenKind::ImportScope => Some(CodedIndex((index << 5) | 0x1A)),
				_ => None,
			},
		}
	}
}
