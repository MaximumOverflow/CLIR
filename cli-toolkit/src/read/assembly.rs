use std::rc::Rc;

pub struct Assembly {
	pub(super) name: String,
	pub(super) culture: String,
	pub(super) version: AssemblyVersion,
	pub(super) dependencies: Vec<Rc<Assembly>>,
}

#[derive(Debug, Clone)]
pub struct AssemblyVersion {
	pub major: u16,
	pub minor: u16,
	pub build: u16,
	pub revision: u16,
}
