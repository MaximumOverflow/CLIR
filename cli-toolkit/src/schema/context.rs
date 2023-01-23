use crate::schema::assembly::Assembly;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context<'l> {
	pub(crate) assembly_vec: Vec<Assembly<'l>>,
	pub(crate) assembly_map: HashMap<String, usize>,
}
