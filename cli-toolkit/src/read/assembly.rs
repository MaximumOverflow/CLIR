use std::fmt::{Debug, Display, Formatter};
use crate::raw::{MetadataToken};
use std::collections::HashMap;
use derivative::Derivative;
use crate::read::Type;
use std::rc::Rc;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Assembly {
	pub(super) name: String,
	pub(super) culture: String,
	#[derivative(Debug(format_with = "Display::fmt"))]
	pub(super) version: AssemblyVersion,
	#[derivative(Debug(format_with = "format_deps"))]
	pub(super) dependencies: Vec<Rc<Assembly>>,
	#[derivative(Debug(format_with = "format_types"))]
	pub(super) types: HashMap<MetadataToken, Rc<Type>>,
}

#[derive(Debug, Clone)]
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

fn format_deps(deps: &Vec<Rc<Assembly>>, f: &mut Formatter) -> Result<(), std::fmt::Error> {
	let deps = deps
		.iter()
		.map(|d| format!("{} {}", d.name, d.version))
		.collect::<Vec<_>>();

	deps.fmt(f)
}

fn format_types(types: &HashMap<MetadataToken, Rc<Type>>, f: &mut Formatter) -> Result<(), std::fmt::Error> {
	types.values().fmt(f)
}
