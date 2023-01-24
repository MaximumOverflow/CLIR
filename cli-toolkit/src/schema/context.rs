use crate::schema::assembly::Assembly;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Context {
	pub(crate) assembly_vec: Vec<Rc<Assembly>>,
	pub(crate) assembly_map: HashMap<String, usize>,
}
