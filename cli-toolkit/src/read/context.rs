use crate::read::assembly::AssemblyReader;
use crate::schema::{Assembly, Context};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::raw::AlignedBuffer;
use std::iter::repeat_with;
use crate::read::Error;
use std::pin::Pin;
use std::rc::Rc;
use crate::utilities::get_mut_unchecked;

pub struct ContextReader<'l> {
	context: Rc<Context>,
	readers: Vec<AssemblyReader<'l>>,
}

impl Context {
	pub fn from_assembly_list<'l, T: TryInto<AlignedBuffer<'l>>>(
		assemblies: impl IntoIterator<Item = T>,
	) -> Result<Rc<Context>, Error>
	where
		Error: From<<T as TryInto<AlignedBuffer<'l>>>::Error>,
	{
		let mut readers = vec![];
		for i in assemblies {
			readers.push(AssemblyReader::new(i.try_into()?)?)
		}

		let reader = ContextReader {
			readers,
			context: Rc::new(Context::default()),
		};

		reader.read()
	}

	pub(crate) fn default() -> Self {
		Self {
			assembly_vec: vec![],
			assembly_map: HashMap::default(),
		}
	}
}

impl<'l> ContextReader<'l> {
	fn read(mut self) -> Result<Rc<Context>, Error> {
		let mut_context = unsafe { get_mut_unchecked(&self.context) };
		mut_context.assembly_vec = Vec::with_capacity(self.readers.len());

		for (index, reader) in self.readers.iter().enumerate() {
			let ident = reader.get_ident()?;
			mut_context.assembly_map.insert(ident, index);
		}

		for reader in self.readers.iter() {
			let mut assembly = Rc::new(Assembly::default());
			let assembly = reader.read_assembly_definition(assembly)?;
			mut_context.assembly_vec.push(assembly);
		}

		for (reader, assembly) in self.readers.iter().zip(mut_context.assembly_vec.iter().cloned()) {
			{
				let mut_assembly = unsafe { get_mut_unchecked(&assembly) };
				mut_assembly.ctx = Rc::downgrade(&self.context);

				reader.read_assembly_refs(mut_assembly);
				reader.read_assembly_type_refs(mut_assembly);
			}
			reader.read_assembly_types(assembly);
		}

		Ok(self.context)
	}
}
