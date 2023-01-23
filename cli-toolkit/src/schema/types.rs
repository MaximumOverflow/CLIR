use crate::raw::{MetadataToken, TableIndex, TypeFlags};
use crate::schema::assembly::Assembly;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum Type<'l> {
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
	Enum(TypeData<'l>),
	Class(TypeData<'l>),
	Struct(TypeData<'l>),
	Interface(TypeData<'l>),
	
	NotLoaded(MetadataToken),
	CustomUnknown(TypeData<'l>),
}

pub struct TypeData<'l> {
	pub(crate) assembly: &'l Assembly<'l>,
	
	pub(crate) name: String,
	pub(crate) namespace: String,
	pub(crate) flags: TypeFlags,
	pub(crate) base: MetadataToken,
	pub(crate) token: MetadataToken,
	pub(crate) fields: Vec<TableIndex>,
}

pub struct Field<'l> {
	parent: &'l Type<'l>,
}

impl Type<'_> {
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

impl Display for Type<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::Class(data) => Display::fmt(data, f),
			Type::Struct(data) => Display::fmt(data, f),
			Type::Interface(data) => Display::fmt(data, f),
			_ => Debug::fmt(self, f),
		}
	}
}

impl Display for TypeData<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self.namespace.is_empty() {
			true => f.write_str(&self.name),
			false => write!(f, "\"{}.{}\"", self.namespace, self.name),
		}
	}
}

impl Debug for TypeData<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let missing = &Type::NotLoaded(self.base);
		let base = self.assembly.get_type(self.base).unwrap_or(missing);
		
		f.debug_struct("TypeData")
			.field("token", &self.token)
			.field("name", &self.name)
			.field("namespace", &self.namespace)
			.field("flags", &format_args!("0x{:X}", self.flags))
			.field("base", &format_args!("{}", base))
			.finish()
	}
}