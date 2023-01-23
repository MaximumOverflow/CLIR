use crate::raw;
use crate::raw::{BlobHeap, CodedIndexKind, FieldTable, MetadataToken, MetadataTokenKind, StringHeap, TableHeap, TypeDef};
use crate::schema::{Assembly, Type, TypeData};
use crate::read::Error;

pub struct TypeReader<'l> {
	index: usize,
	ty: &'l mut Type<'l>,

	def: TypeDef,
	next: Option<TypeDef>,

	blobs: BlobHeap<'l>,
	tables: TableHeap<'l>,
	strings: StringHeap<'l>,
	assembly: &'l Assembly<'l>,
}

impl<'l> Type<'l> {
	pub(crate) fn default() -> Self {
		Self::Void
	}

	pub(crate) fn read(
		index: usize,
		ty: &'l mut Type<'l>,
		def: TypeDef,
		next: Option<TypeDef>,

		blobs: BlobHeap<'l>,
		tables: TableHeap<'l>,
		strings: StringHeap<'l>,
		assembly: &'l Assembly<'l>,
	) -> TypeReader<'l> {
		TypeReader {
			index,
			ty,
			def,
			next,
			blobs,
			tables,
			strings,
			assembly,
		}
	}
}

impl<'l> TypeReader<'l> {
	pub(crate) fn initialize(&mut self) -> Result<(), Error> {
		let mut data = TypeData {
			fields: vec![],
			base: MetadataToken(0),
			assembly: self.assembly,
			flags: self.def.flags(),
			name: self.strings.get_string(self.def.name()).to_string(),
			namespace: self.strings.get_string(self.def.namespace()).to_string(),
			token: MetadataToken::new(self.index as u32 + 1, MetadataTokenKind::TypeDef),
		};

		*self.ty = Type::Class(data);

		Ok(())
	}

	pub(crate) fn populate(&mut self) -> Result<(), Error> {
		let Type::Class(data) = self.ty else { unreachable!() };
		let data = std::mem::replace(
			data,
			TypeData {
				assembly: self.assembly,
				name: "".to_string(),
				namespace: "".to_string(),
				flags: 0,
				base: MetadataToken(0),
				token: MetadataToken(0),
				fields: vec![],
			},
		);

		self.read_base(data)
	}

	fn read_base(&mut self, mut data: TypeData<'l>) -> Result<(), Error> {
		let token = self
			.def
			.extends()
			.decode(CodedIndexKind::TypeDefOrRef)
			.ok_or(raw::Error::InvalidData(Some("Invalid field base type")))?;

		if data.name == "Program" {
			println!();
		}

		data.base = token;
		match self.assembly.get_type(token) {
			Some(base) => {
				*self.ty = match base {
					Type::Class(base) => match (base.namespace.as_str(), base.name.as_str()) {
						("System", "ValueType") => Type::Struct(data),
						_ => Type::Class(data),
					},

					Type::Struct(base) => match (base.namespace.as_str(), base.name.as_str()) {
						("System", "Enum") => Type::Struct(data),
						base => unimplemented!("{:?}", base),
					},

					Type::CustomUnknown(base) => Type::CustomUnknown(data),

					base => unimplemented!("{:?}", base),
				}
			}

			None => {
				if data.namespace == "System" && data.name == "Object" {
					*self.ty = Type::Class(data);
					return Ok(());
				}

				if data.flags & raw::type_flags::INTERFACE != 0 {
					*self.ty = Type::Interface(data);
					return Ok(());
				}

				*self.ty = Type::CustomUnknown(data)
			}
		};

		Ok(())
	}

	fn read_fields(&self, data: &mut TypeData) -> Result<(), Error> {
		let field_table = match self.tables.get_table::<FieldTable>()? {
			Some(table) => table,
			None => return Ok(()),
		};

		unimplemented!()
	}
}
