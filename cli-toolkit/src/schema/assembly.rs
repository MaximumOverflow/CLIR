use crate::raw::{AssemblyFlags, MetadataToken, MetadataTokenKind};
use std::fmt::{Debug, Display, Formatter};
use crate::schema::context::Context;
use crate::schema::types::TypeData;
use crate::schema::Type;

pub struct Assembly<'l> {
	pub(crate) ctx: &'l Context<'l>,
	pub(crate) name: AssemblyName,

	pub(crate) types: Vec<Type<'l>>,
	pub(crate) type_refs: Vec<(MetadataToken, String, String)>,

	pub(crate) fields: Vec<TypeData<'l>>,
	pub(crate) dependencies: Vec<AssemblyRef>,
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

impl<'l> Assembly<'l> {
	pub fn find_type(&'l self, name: &str, namespace: &str) -> Option<&'l Type<'l>> {
		if let Some(ty) = self.types.iter().find(|ty| ty.matches_name(name, namespace)) {
			return Some(ty);
		}

		for assembly in self.dependencies.iter() {
			let Some(assembly) = self.ctx.assembly_map.get(&assembly.ident_key) else { continue };
			let Some(assembly) = self.ctx.assembly_vec.get(*assembly) else { continue };

			if let Some(ty) = assembly.find_type(name, namespace) {
				return Some(ty);
			}
		}

		None
	}

	pub fn get_type(&'l self, token: MetadataToken) -> Option<&'l Type<'l>> {
		match token.token_kind() {
			MetadataTokenKind::TypeDef => self.types.get((token.index() - 1) as usize),
			MetadataTokenKind::TypeRef => {
				let (token, namespace, name) = &self.type_refs.get(token.index() - 1)?;
				match token.token_kind() {
					MetadataTokenKind::AssemblyRef => {
						let assembly_ref = &self.dependencies.get(token.index() - 1)?;
						let assembly = self.ctx.assembly_map.get(&assembly_ref.ident_key)?;
						let assembly = self.ctx.assembly_vec.get(*assembly)?;
						assembly.find_type(&name, &namespace)
					}
					_ => unimplemented!("{:?}", token.token_kind()),
				}
			}
			_ => None,
		}
	}
}

impl Debug for Assembly<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Assembly")
			.field("name", &self.name)
			.field("dependencies", &Deps(&self.dependencies))
			.field("types", &self.types)
			.finish()
	}
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
