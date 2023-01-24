use std::cell::RefCell;
use crate::raw::{MetadataToken, TableIndex, TypeFlags};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use crate::schema::assembly::Assembly;
use crate::utilities::IndexedRcRef;
use std::rc::{Rc, Weak};
use bitvec::mem::elts;

#[derive(Debug)]
pub enum Type {
	Void,
	Char,
	Int8,
	Int16,
	Int32,
	Int64,
	UInt8,
	UInt16,
	UInt32,
	UInt64,
	Float,
	Double,
	Object,
	String,
	Enum(TypeData),
	Class(TypeData),
	Struct(TypeData),
	Interface(TypeData),

	Uninitialized(TypeData),
	CustomUnknown(TypeData),
	NotLoaded(MetadataToken),
}

impl Type {
	pub(crate) fn matches_name(&self, name: &str, namespace: &str) -> bool {
		let (ty_name, ty_namespace) = match self {
			Type::String => ("String", "System"),
			Type::Object => ("Object", "System"),
			Type::Enum(data) => (data.name.as_str(), data.namespace.as_str()),
			Type::Class(data) => (data.name.as_str(), data.namespace.as_str()),
			Type::Struct(data) => (data.name.as_str(), data.namespace.as_str()),
			Type::Interface(data) => (data.name.as_str(), data.namespace.as_str()),
			Type::CustomUnknown(data) => (data.name.as_str(), data.namespace.as_str()),
			_ => return false,
		};

		ty_name == name && ty_namespace == namespace
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::Class(data) => Display::fmt(data, f),
			Type::Struct(data) => Display::fmt(data, f),
			Type::Interface(data) => Display::fmt(data, f),
			_ => Debug::fmt(self, f),
		}
	}
}

pub type TypeRef = IndexedRcRef<Type, [Type]>;

pub struct TypeData {
	pub(crate) assembly: Weak<Assembly>,

	pub(crate) name: String,
	pub(crate) namespace: String,
	pub(crate) flags: TypeFlags,
	pub(crate) base: MetadataToken,
	pub(crate) token: MetadataToken,
	pub(crate) fields: Vec<TableIndex>,
}

impl Display for TypeData {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.namespace.is_empty() {
			true => f.write_str(&self.name),
			false => write!(f, "\"{}.{}\"", self.namespace, self.name),
		}
	}
}

impl Debug for TypeData {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let Some(assembly) = self.assembly.upgrade() else {
			return f.write_str("null");
		};

		let missing = &Type::NotLoaded(self.base);
		let base = assembly.get_type(self.base);

		let mut f = f.debug_struct("TypeData");
		f.field("token", &self.token);
		f.field("name", &self.name);
		f.field("namespace", &self.namespace);
		f.field("flags", &format_args!("0x{:X}", self.flags));

		let base = match base {
			None => f.field("base", &format_args!("{}", missing)),
			Some(ty) => f.field("base", &format_args!("{}", ty.deref())),
		};

		f.finish()
	}
}

pub struct Field {
	pub(crate) assembly: Weak<Assembly>,
	pub(crate) parent: MetadataToken,
}
