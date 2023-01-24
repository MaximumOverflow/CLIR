use crate::raw::{AssemblyFlags, MetadataToken, MetadataTokenKind};
use std::fmt::{Debug, Display, Formatter};
use crate::schema::context::Context;
use crate::schema::types::TypeData;
use crate::utilities::IndexedRcRef;
use std::cell::{Ref, RefCell};
use std::marker::PhantomData;
use std::ops::Deref;
use crate::schema::{Type, TypeRef};
use std::rc::{Rc, Weak};

pub struct Assembly {
	pub(crate) ctx: Weak<Context>,

	pub(crate) name: AssemblyName,

	pub(crate) types: Rc<[Type]>,
	pub(crate) dependencies: Vec<AssemblyRef>,
	pub(crate) type_refs: Vec<(MetadataToken, String, String)>,
}

impl Debug for Assembly {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Assembly")
			.field("name", &self.name)
			.field("dependencies", &Deps(&self.dependencies))
			.field("types", &self.types)
			.finish()
	}
}

impl Assembly {
	pub fn find_type(&self, name: &str, namespace: &str) -> Option<TypeRef> {
		if let Some(ty) = self.types.iter().find(|ty| ty.matches_name(name, namespace)) {
			match ty {
				Type::Enum(data)
				| Type::Class(data)
				| Type::Struct(data)
				| Type::Interface(data)
				| Type::CustomUnknown(data) => {
					let index = data.token.index() - 1;
					return Some(TypeRef::new(self.types.clone(), index));
				}
				_ => unimplemented!(),
			}
		}

		for assembly in self.dependencies.iter() {
			let ctx = self.ctx.upgrade().unwrap();
			let Some(assembly) = ctx.assembly_map.get(&assembly.ident_key) else { continue };
			let Some(assembly) = ctx.assembly_vec.get(*assembly) else { continue };

			let assembly = assembly.clone();
			if let Some(ty) = assembly.find_type(name, namespace) {
				return Some(ty);
			}
		}

		None
	}

	pub fn get_type(&self, token: MetadataToken) -> Option<TypeRef> {
		let ctx = self.ctx.upgrade().unwrap();
		get_type(token, &ctx, &self.types, &self.dependencies, &self.type_refs)
	}
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct AssemblyVersion {
	pub major: u16,
	pub minor: u16,
	pub build: u16,
	pub revision: u16,
}

impl Display for AssemblyVersion {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}.{}.{}.{}", self.major, self.minor, self.build, self.revision)
	}
}

pub(crate) struct AssemblyName {
	pub(crate) name: String,
	pub(crate) culture: String,
	pub(crate) version: AssemblyVersion,
	pub(crate) flags: AssemblyFlags,
	pub(crate) public_key: Vec<u8>,
}

impl Display for AssemblyName {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} {}", self.name, self.version)
	}
}

impl Debug for AssemblyName {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("AssemblyName")
			.field("name", &self.name)
			.field("culture", &self.culture)
			.field("version", &self.version)
			.field("flags", &format_args!("0x{:X}", self.flags))
			.field("public_key", &format_args!("{:?}", self.public_key))
			.finish()
	}
}

#[derive(Debug)]
pub(crate) struct AssemblyRef {
	pub(crate) name: String,
	pub(crate) culture: String,
	pub(crate) version: AssemblyVersion,
	pub(crate) flags: AssemblyFlags,
	pub(crate) public_key: Vec<u8>,
	pub(crate) hash_value: Vec<u8>,
	pub(crate) ident_key: String,
}

struct Deps<'l>(&'l [AssemblyRef]);

impl Debug for Deps<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let mut list = f.debug_list();
		for assembly in self.0 {
			list.entry(&format_args!("\"{} {}\"", assembly.name, assembly.version));
		}

		list.finish()
	}
}

pub(crate) fn get_type(
	token: MetadataToken,
	ctx: &Context,
	types: &Rc<[Type]>,
	dependencies: &[AssemblyRef],
	type_refs: &[(MetadataToken, String, String)],
) -> Option<TypeRef> {
	match token.token_kind() {
		MetadataTokenKind::TypeDef => {
			let index = token.index() - 1;
			match index < types.len() {
				true => Some(TypeRef::new(types.clone(), index)),
				false => None,
			}
		}

		MetadataTokenKind::TypeRef => {
			let (token, namespace, name) = &type_refs.get(token.index() - 1)?;
			match token.token_kind() {
				MetadataTokenKind::AssemblyRef => {
					let assembly_ref = dependencies.get(token.index() - 1)?;
					let assembly = ctx.assembly_map.get(&assembly_ref.ident_key)?;
					let assembly = ctx.assembly_vec.get(*assembly)?;
					assembly.find_type(&name, &namespace)
				}
				_ => unimplemented!("{:?}", token.token_kind()),
			}
		}
		_ => None,
	}
}
