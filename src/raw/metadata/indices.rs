use crate::raw::*;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct MetadataIndex(pub(crate) usize);

#[derive(Debug, Copy, Clone)]
pub enum MetadataIndexSize {
	Slim = 0x2,
	Fat = 0x4,
}

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

pub(crate) fn get_coded_index_size(kind: CodedIndexKind, tables_heap: &TableHeap) -> MetadataIndexSize {
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
		true => MetadataIndexSize::Slim,
		false => MetadataIndexSize::Fat,
	}
}
