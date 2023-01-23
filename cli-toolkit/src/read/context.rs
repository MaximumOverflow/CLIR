use crate::schema::{Assembly, Context};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::iter::repeat_with;
use crate::read::Error;

pub struct ContextReader<'l> {
	paths: Vec<PathBuf>,
	context: Box<Context<'l>>,
}

impl Context<'_> {
	pub fn from_assembly_list<T: AsRef<Path>>(assemblies: impl IntoIterator<Item = T>) -> Result< Box<Self>, Error> {
		Self::read(assemblies.into_iter().map(|e| PathBuf::from(e.as_ref())).collect())
	}

	pub(crate) fn default() -> Self {
		Self {
			assembly_vec: vec![],
			assembly_map: HashMap::default(),
		}
	}

	pub(crate) fn read<'l>(paths: Vec<PathBuf>) -> Result< Box<Self>, Error> {
		let reader = ContextReader { 
			paths,
			context: Box::new(Context::default()) 
		};
		
		reader.read()
	}
}

impl <'l> ContextReader<'l> {
	fn read(mut self) -> Result<Box<Context<'l>>, Error> {
		self.context.assembly_vec = repeat_with(Assembly::default).take(self.paths.len()).collect();
		
		let assemblies = unsafe {
			let ptr = self.context.assembly_vec.as_mut_ptr();
			(0..self.context.assembly_vec.len()).map(move |i| {
				let ass: &'l mut Assembly<'l> = std::mem::transmute(&mut *ptr.add(i));
				ass
			})
		};
		
		for (index, (assembly, path)) in assemblies.zip(self.paths).enumerate() {
			assembly.ctx = unsafe { &*(self.context.as_ref() as *const Context<'l>) };
			let reader = Assembly::read(assembly, path)?;
			
			let ident = reader.get_ident()?;
			self.context.assembly_map.insert(ident, index);

			let _ = reader.read()?;
		}
		
		Ok(self.context)
	}
}