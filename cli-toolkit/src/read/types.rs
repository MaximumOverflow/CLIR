use crate::raw::{
	BlobHeap, CodedIndexKind, FieldTable, MetadataTable, MetadataToken, MetadataTokenKind, StringHeap, TableHeap,
	TableIndex, type_flags, TypeDef, TypeDefTable,
};
use crate::schema::{Assembly, get_type, Type, TypeData};
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::read::{Error, types};
use std::ops::{Deref, DerefMut};
use std::ptr::null;
use bitvec::index;
use crate::raw;

pub struct TypeReader<'l> {
	blobs: BlobHeap<'l>,
	tables: TableHeap<'l>,
	assembly: Rc<Assembly>,
	strings: StringHeap<'l>,
	type_defs: TypeDefTable<'l>,
}

impl Type {
	pub(crate) fn default() -> Self {
		Self::Void
	}

	pub(crate) fn read<'l>(
		blobs: BlobHeap<'l>,
		tables: TableHeap<'l>,
		strings: StringHeap<'l>,
		type_defs: TypeDefTable<'l>,
		assembly: Rc<Assembly>,
	) -> TypeReader<'l> {
		TypeReader {
			blobs,
			tables,
			strings,
			type_defs,
			assembly,
		}
	}
}

impl TypeData {
	pub(crate) fn default() -> TypeData {
		Self {
			assembly: Weak::new(),
			name: "".to_string(),
			namespace: "".to_string(),
			flags: 0,
			base: MetadataToken(0),
			token: MetadataToken(0),
			fields: vec![],
		}
	}
}

impl<'l> TypeReader<'l> {
	pub(crate) fn read_type_definition(&self, index: usize, types: &mut Rc<[Type]>) -> Result<(), Error> {
		let metadata_index = (index + 1) as u32;
		let def = self.type_defs.get(TableIndex(metadata_index))?;

		let base = def
			.base_type()
			.decode(CodedIndexKind::TypeDefOrRef)
			.ok_or(raw::Error::InvalidData(Some("Invalid field base type")))?;

		let types = Rc::get_mut(types).unwrap();
		types[index] = Type::Uninitialized(TypeData {
			base,
			fields: vec![],
			flags: def.flags(),
			assembly: Rc::downgrade(&self.assembly),
			name: self.strings.get_string(def.name()).to_string(),
			namespace: self.strings.get_string(def.namespace()).to_string(),
			token: MetadataToken::new(metadata_index, MetadataTokenKind::TypeDef),
		});

		Ok(())
	}

	pub(crate) fn read_base(&self, index: usize, types: &mut Rc<[Type]>) -> Result<(), Error> {
		let data = {
			let types = Rc::get_mut(types).unwrap();
			let mut ty = &mut types[index];

			let Type::Uninitialized(data) = ty else { return Ok(()) };
			std::mem::replace(data, TypeData::default())
		};

		let ctx = self.assembly.ctx.upgrade().unwrap();
		let dependencies = &self.assembly.dependencies;
		let type_refs = &self.assembly.type_refs;

		macro_rules! set_ty {
			($idx: expr, $types: expr, $val: expr) => {
				set_ty!($idx, $types, $val, 0)
			};

			($idx: expr, $types: expr, $val: expr, $base: expr) => {{
				drop($base);
				let types = Rc::get_mut($types).unwrap();
				types[$idx] = $val;
				Ok(())
			}};
		}

		if data.base.is_null() {
			if data.flags & type_flags::INTERFACE != 0 {
				return set_ty!(index, types, Type::Interface(data));
			}

			match (data.namespace.as_str(), data.name.as_str(), data.flags) {
				("System", "Object", 0x102001) => {
					return set_ty! {
						index,
						types,
						Type::Class(data)
					}
				}
				("", "<Module>", 0x0) => {
					return set_ty! {
						index,
						types,
						Type::CustomUnknown(data)
					}
				}
				_ => unimplemented!("{:?}", data),
			}
		}

		loop {
			match get_type(data.base, &ctx, types, &dependencies, type_refs) {
				Some(base_ref) => {
					let base = base_ref.deref();
					match base {
						Type::Class(base) => match (base.namespace.as_str(), base.name.as_str(), base.flags) {
							("System", "ValueType", 0x102081) => {
								return set_ty! {
									index,
									types,
									Type::Struct(data),
									base_ref
								}
							}

							_ => {
								return set_ty! {
									index,
									types,
									Type::Class(data),
									base_ref
								}
							}
						},

						Type::Uninitialized(base) => match base.token.token_kind() {
							MetadataTokenKind::TypeDef => {
								let index = base.token.index() - 1;

								drop(base_ref);
								self.read_base(index, types);
							}

							_ => unimplemented!("{:?}", base),
						},

						Type::CustomUnknown(_) => {
							return set_ty! {
								index,
								types,
								Type::CustomUnknown(data),
								base_ref
							}
						}

						Type::Struct(base) => match (base.namespace.as_str(), base.name.as_str(), base.flags) {
							("System", "Enum", 0x102081) => {
								return set_ty! {
									index,
									types,
									Type::Enum(data),
									base_ref
								}
							}

							_ => unimplemented!("{:?}", base),
						},

						_ => unimplemented!("{:?}", base),
					}
				}

				None => {
					return set_ty! {
						index,
						types,
						Type::CustomUnknown(data)
					}
				}
			}
		}
	}
}
