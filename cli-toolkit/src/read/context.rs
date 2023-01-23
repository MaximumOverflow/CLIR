use crate::schema::{Assembly, Context};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::raw::AlignedBuffer;
use std::iter::repeat_with;
use crate::read::Error;

pub struct ContextReader<'l> {
	buffers: Vec<AlignedBuffer<'l>>,
	context: Box<Context<'l>>,
}

impl<'l> Context<'l> {
	pub fn from_assembly_list<T: TryInto<AlignedBuffer<'l>>>(
		assemblies: impl IntoIterator<Item = T>,
	) -> Result<Box<Self>, Error>
	where
		Error: From<<T as TryInto<AlignedBuffer<'l>>>::Error>,
	{
		let mut buffers = vec![];
		for i in assemblies {
			buffers.push(i.try_into()?);
		}
		Self::read(buffers)
	}

	pub(crate) fn default() -> Self {
		Self {
			assembly_vec: vec![],
			assembly_map: HashMap::default(),
		}
	}

	pub(crate) fn read(buffers: Vec<AlignedBuffer<'l>>) -> Result<Box<Self>, Error> {
		let reader = ContextReader {
			buffers,
			context: Box::new(Context::default()),
		};

		reader.read()
	}
}

impl<'l> ContextReader<'l> {
	fn read(mut self) -> Result<Box<Context<'l>>, Error> {
		self.context.assembly_vec = repeat_with(Assembly::default).take(self.buffers.len()).collect();

		let assemblies = unsafe {
			let ptr = self.context.assembly_vec.as_mut_ptr();
			(0..self.context.assembly_vec.len()).map(move |i| {
				let ass: &'l mut Assembly<'l> = std::mem::transmute(&mut *ptr.add(i));
				ass
			})
		};

		for (index, (assembly, bytes)) in assemblies.zip(self.buffers).enumerate() {
			assembly.ctx = unsafe { &*(self.context.as_ref() as *const Context<'l>) };
			let reader = Assembly::read(assembly, bytes)?;

			let ident = reader.get_ident()?;
			self.context.assembly_map.insert(ident, index);

			let _ = reader.read()?;
		}

		Ok(self.context)
	}
}
