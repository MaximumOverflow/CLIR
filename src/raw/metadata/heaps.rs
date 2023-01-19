use std::fmt::{Debug, Formatter};
use bitvec::array::BitArray;
use strum::IntoEnumIterator;
use std::mem::size_of;
use crate::raw::*;
use indoc::indoc;
use uuid::Uuid;

pub struct StringHeap<'l> {
	bytes: &'l [u8],
}

impl<'l> MetadataHeap<'l> for StringHeap<'l> {
	fn new(bytes: &'l [u8]) -> Self {
		Self { bytes }
	}
	fn cli_identifier() -> &'static str {
		"#Strings"
	}
	fn idx_size(tables: &TableHeap) -> MetadataIndexSize {
		match (tables.heap_sizes().data[0] & 0x1) != 0 {
			true => MetadataIndexSize::Fat,
			false => MetadataIndexSize::Slim,
		}
	}
}

impl StringHeap<'_> {
	pub fn get_string_at(&self, index: MetadataIndex) -> &str {
		let bytes = &self.bytes[index.0..];
		let bytes = &bytes[..bytes.iter().position(|c| *c == 0).unwrap_or(bytes.len())];
		std::str::from_utf8(bytes).unwrap()
	}
}

impl Debug for StringHeap<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		unsafe { write!(f, "{:?}", std::str::from_utf8_unchecked(self.bytes)) }
	}
}

pub struct GuidHeap<'l> {
	bytes: &'l [u8],
}

impl<'l> MetadataHeap<'l> for GuidHeap<'l> {
	fn new(bytes: &'l [u8]) -> Self {
		Self { bytes }
	}
	fn cli_identifier() -> &'static str {
		"#GUID"
	}
	fn idx_size(tables: &TableHeap) -> MetadataIndexSize {
		match (tables.heap_sizes().data[0] & 0x2) != 0 {
			true => MetadataIndexSize::Fat,
			false => MetadataIndexSize::Slim,
		}
	}
}

impl Debug for GuidHeap<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		unsafe {
			let count = self.bytes.len() / size_of::<Uuid>();
			let guids = std::slice::from_raw_parts(self.bytes.as_ptr() as *const Uuid, count);
			guids.fmt(f)
		}
	}
}

pub struct BlobHeap<'l> {
	bytes: &'l [u8],
}

impl<'l> MetadataHeap<'l> for BlobHeap<'l> {
	fn new(bytes: &'l [u8]) -> Self {
		Self { bytes }
	}
	fn cli_identifier() -> &'static str {
		"#Blob"
	}
	fn idx_size(tables: &TableHeap) -> MetadataIndexSize {
		match (tables.heap_sizes().data[0] & 0x4) != 0 {
			true => MetadataIndexSize::Fat,
			false => MetadataIndexSize::Slim,
		}
	}
}

impl Debug for BlobHeap<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "BlobHeap([u8; {}])", self.bytes.len())
	}
}

pub struct UserStringHeap<'l> {
	bytes: &'l [u8],
}

impl<'l> MetadataHeap<'l> for UserStringHeap<'l> {
	fn new(bytes: &'l [u8]) -> Self {
		Self { bytes }
	}
	fn cli_identifier() -> &'static str {
		"#US"
	}
	fn idx_size(_: &TableHeap) -> MetadataIndexSize {
		unimplemented!()
	}
}

impl Debug for UserStringHeap<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		unsafe { write!(f, "{:?}", std::str::from_utf8_unchecked(self.bytes)) }
	}
}

#[derive(Clone)]
pub struct TableHeap<'l> {
	bytes: &'l [u8],
}

impl<'l> MetadataHeap<'l> for TableHeap<'l> {
	fn new(bytes: &'l [u8]) -> Self {
		Self { bytes }
	}
	fn cli_identifier() -> &'static str {
		"#~"
	}
	fn idx_size(_: &TableHeap) -> MetadataIndexSize {
		unimplemented!()
	}
}

impl<'l> TableHeap<'l> {
	pub fn major_version(&self) -> u8 {
		self.bytes[4]
	}

	pub fn minor_version(&self) -> u8 {
		self.bytes[5]
	}

	pub fn heap_sizes(&self) -> BitArray<[u8; 1]> {
		BitArray::new([self.bytes[6]])
	}

	pub fn valid(&self) -> BitArray<[u64; 1]> {
		let mut valid = [0; 8];
		valid.copy_from_slice(&self.bytes[8..16]);
		BitArray::new([u64::from_le_bytes(valid)])
	}

	pub fn sorted(&self) -> BitArray<[u64; 1]> {
		let mut valid = [0; 8];
		valid.copy_from_slice(&self.bytes[16..24]);
		BitArray::new([u64::from_le_bytes(valid)])
	}

	pub fn table_count(&self) -> usize {
		self.valid().count_ones()
	}

	pub fn has_table(&self, kind: TableKind) -> bool {
		self.valid().get(kind as usize).as_deref().cloned().unwrap_or(false)
	}

	pub fn get_table<T: MetadataTable<'l>>(&self) -> Result<Option<T>, Error> {
		let mut reader = ByteStream::new(self.bytes);
		reader.skip(24 + 4 * self.table_count())?;

		let rows = self.rows();
		let indices = 0..self.table_count();
		let tables = TableKind::iter().filter(|k| self.has_table(*k));

		for (index, table) in indices.zip(tables) {
			let rows = rows[index] as usize;
			let row_size = self.row_size(table);
			let table_size = rows * row_size;

			if table == T::cli_identifier() {
				let bytes = reader.read_slice::<u8>(table_size)?;
				return Ok(Some(T::new(bytes, self)?));
			} else {
				reader.skip(table_size)?;
			}
		}

		Ok(None)
	}

	pub fn rows(&self) -> &[u32] {
		let count = self.table_count();
		let bytes = &self.bytes[24..24 + 4 * count];
		unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u32, count) }
	}

	pub fn row_count(&self, table: TableKind) -> usize {
		if !self.has_table(table) {
			return 0;
		}

		let mut index = 0;
		for kind in TableKind::iter() {
			if kind == table {
				break;
			} else {
				index += self.has_table(kind) as usize;
			}
		}

		return self.rows()[index] as usize;
	}

	pub fn row_size(&self, table: TableKind) -> usize {
		match table {
			TableKind::Field => FieldTable::row_size(self),
			TableKind::Module => ModuleTable::row_size(self),
			TableKind::TypeRef => TypeRefTable::row_size(self),
			TableKind::TypeDef => TypeDefTable::row_size(self),
			TableKind::Assembly => AssemblyTable::row_size(self),
			_ => unimplemented!("Unimplemented table {:?}", table),
		}
	}

	pub fn table_idx_size(&self, table: TableKind) -> MetadataIndexSize {
		match self.row_count(table) <= u16::MAX as usize {
			true => MetadataIndexSize::Slim,
			false => MetadataIndexSize::Fat,
		}
	}
}

impl Debug for TableHeap<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if f.alternate() {
			write!(
				f,
				indoc! {
					"TableHeap {{
						major_version: {},
						minor_version: {},
						heap_sizes:    {:b},
						valid:         {:b},
						sorted:        {:b},
						rows:          {:?},
					}}"
				},
				self.major_version(),
				self.minor_version(),
				self.heap_sizes(),
				self.valid(),
				self.sorted(),
				self.rows(),
			)
		} else {
			write!(f, "TableHeap {{ ")?;
			write!(f, "major_version: {}, ", self.major_version())?;
			write!(f, "minor_version: {}, ", self.minor_version())?;
			write!(f, "heap_sizes: 0b{:b}, ", self.heap_sizes())?;
			write!(f, "valid: 0b{:b}, ", self.valid())?;
			write!(f, "sorted: 0b{:b}, ", self.sorted())?;
			write!(f, "rows: {:?}, ", self.rows())?;
			write!(f, "}}")?;
			Ok(())
		}
	}
}

pub(crate) mod private {
	use crate::raw::*;
	pub trait MetadataHeap<'l> {
		fn new(bytes: &'l [u8]) -> Self;
		fn cli_identifier() -> &'static str;
		fn idx_size(tables: &TableHeap) -> MetadataIndexSize;
	}
}
