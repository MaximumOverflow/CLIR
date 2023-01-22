use crate::raw::{
	BlobHeap, FieldTable, MetadataTable, MetadataToken, MetadataTokenKind, StringHeap, TableHeap, TableIndex,
	TypeDefTable,
};
use crate::read::{Error, try_get_table};
use std::collections::HashMap;
use derivative::Derivative;
use std::rc::Rc;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Type {
	token: MetadataToken,
	name: String,
	namespace: String,
	fields: Vec<Rc<Field>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TypeKind {
	Class,
	Struct,
	Interface,
	Primitive,
}

#[derive(Debug)]
pub struct Field {
	token: MetadataToken,
	name: String,
}

impl Type {
	pub fn token(&self) -> MetadataToken {
		self.token
	}

	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn namespace(&self) -> &str {
		&self.namespace
	}

	pub(super) fn read_all(
		blobs: BlobHeap,
		tables: TableHeap,
		strings: StringHeap,
	) -> Result<HashMap<MetadataToken, Rc<Type>>, Error> {
		let mut types = HashMap::new();
		let (type_table, field_table) =
			(try_get_table::<TypeDefTable>(&tables)?, try_get_table::<FieldTable>(&tables)?);

		if let (Some(type_table), Some(field_table)) = (type_table, field_table) {
			let mut next_def = type_table.iter().skip(1);
			for (index, def) in (1..).zip(type_table.iter()) {
				let def = match def {
					Ok(def) => def,
					Err(err) => return Err(Error::ReadError(err)),
				};

				let mut type_rc = Rc::new(Type {
					fields: vec![],
					name: strings.get_string(def.name()).into(),
					namespace: strings.get_string(def.namespace()).into(),
					token: MetadataToken::new(index, MetadataTokenKind::TypeDef),
				});

				let ty = Rc::get_mut(&mut type_rc).unwrap();
				let field_range = match next_def.next() {
					Some(next) => {
						let next = next.map_err(|e| Error::ReadError(e))?;
						def.fields().0..next.fields().0
					}
					None => def.fields().0..field_table.len() as u32 + 1,
				};

				for index in field_range {
					let index = TableIndex(index);
					let def = field_table.get(index).map_err(|e| Error::ReadError(e))?;

					let field = Rc::new(Field {
						token: MetadataToken::new(index.0, MetadataTokenKind::Field),
						name: strings.get_string(def.name()).into(),
					});

					ty.fields.push(field);
				}

				types.insert(ty.token, type_rc);
			}
		}

		Ok(types)
	}
}
