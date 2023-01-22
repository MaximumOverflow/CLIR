use std::fmt::{Debug, Formatter};
use bitvec::array::BitArray;
use strum::IntoEnumIterator;
use std::mem::size_of;
use crate::raw::*;
use indoc::indoc;
use uuid::Uuid;

#[derive(Copy, Clone)]
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
	fn idx_size(tables: &TableHeap) -> IndexSize {
		match (tables.heap_sizes().data[0] & 0x1) != 0 {
			true => IndexSize::Fat,
			false => IndexSize::Slim,
		}
	}
}

impl<'l> StringHeap<'l> {
	pub fn get_string(&self, index: HeapIndex) -> &'l str {
		let bytes = &self.bytes[index.0 as usize..];
		let bytes = &bytes[..bytes.iter().position(|c| *c == 0).unwrap_or(bytes.len())];
		unsafe { std::str::from_utf8_unchecked(bytes) }
	}
}

impl Debug for StringHeap<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		unsafe { write!(f, "{:?}", std::str::from_utf8_unchecked(self.bytes)) }
	}
}

#[derive(Copy, Clone)]
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
	fn idx_size(tables: &TableHeap) -> IndexSize {
		match (tables.heap_sizes().data[0] & 0x2) != 0 {
			true => IndexSize::Fat,
			false => IndexSize::Slim,
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

#[derive(Copy, Clone)]
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
	fn idx_size(tables: &TableHeap) -> IndexSize {
		match (tables.heap_sizes().data[0] & 0x4) != 0 {
			true => IndexSize::Fat,
			false => IndexSize::Slim,
		}
	}
}

impl<'l> BlobHeap<'l> {
	pub fn get_blob(&self, index: MetadataToken) -> Result<&'l [u8], Error> {
		let mut reader = ByteStream::new(self.bytes);
		reader.seek(index.0 as usize)?;

		let length = {
			let byte_0 = reader.read::<u8>()?;
			if byte_0 & 0x80 == 0 {
				(byte_0 & 0x7F) as usize
			} else if byte_0 & 0xC0 == 0x80 {
				let byte_1 = reader.read::<u8>()?;
				(((byte_0 & 0x3F) as usize) << 8) + byte_1 as usize
			} else if byte_0 & 0xE0 == 0xC0 {
				let byte_1 = reader.read::<u8>()?;
				let byte_2 = reader.read::<u8>()?;
				(((byte_0 & 0x3F) as usize) << 16) + ((byte_1 as usize) << 8) + byte_2 as usize
			} else {
				return Err(Error::InvalidData(reader.position() - 1, None));
			}
		};

		reader.read_slice::<u8>(length)
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
	fn idx_size(_: &TableHeap) -> IndexSize {
		unimplemented!()
	}
}

impl Debug for UserStringHeap<'_> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		unsafe { write!(f, "{:?}", std::str::from_utf8_unchecked(self.bytes)) }
	}
}

#[derive(Copy, Clone)]
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
	fn idx_size(_: &TableHeap) -> IndexSize {
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

	pub fn has_table(&self, kind: TableKind) -> bool {
		self.valid().get(kind as usize).as_deref().cloned().unwrap_or(false)
	}

	pub fn get_table<T: MetadataTableImpl<'l>>(&self) -> Result<Option<T>, Error> {
		if !self.has_table(T::cli_identifier()) {
			return Ok(None);
		}

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

	fn heap_sizes(&self) -> BitArray<[u8; 1]> {
		BitArray::new([self.bytes[6]])
	}

	fn valid(&self) -> BitArray<[u64; 1]> {
		let mut valid = [0; 8];
		valid.copy_from_slice(&self.bytes[8..16]);
		BitArray::new([u64::from_le_bytes(valid)])
	}

	fn sorted(&self) -> BitArray<[u64; 1]> {
		let mut valid = [0; 8];
		valid.copy_from_slice(&self.bytes[16..24]);
		BitArray::new([u64::from_le_bytes(valid)])
	}

	fn table_count(&self) -> usize {
		self.valid().count_ones()
	}

	fn rows(&self) -> &[u32] {
		let count = self.table_count();
		let bytes = &self.bytes[24..24 + 4 * count];
		unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u32, count) }
	}

	pub(crate) fn row_count(&self, table: TableKind) -> usize {
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

	fn row_size(&self, table: TableKind) -> usize {
		match table {
			TableKind::Param => ParamTable::calc_row_size(self),
			TableKind::Field => FieldTable::calc_row_size(self),
			TableKind::Event => EventTable::calc_row_size(self),
			TableKind::Module => ModuleTable::calc_row_size(self),
			TableKind::TypeRef => TypeRefTable::calc_row_size(self),
			TableKind::TypeDef => TypeDefTable::calc_row_size(self),
			TableKind::ImplMap => ImplMapTable::calc_row_size(self),
			TableKind::TypeSpec => TypeSpecTable::calc_row_size(self),
			TableKind::Property => PropertyTable::calc_row_size(self),
			TableKind::Assembly => AssemblyTable::calc_row_size(self),
			TableKind::FieldRVA => FieldRVATable::calc_row_size(self),
			TableKind::Constant => ConstantTable::calc_row_size(self),
			TableKind::EventMap => EventMapTable::calc_row_size(self),
			TableKind::MemberRef => MemberRefTable::calc_row_size(self),
			TableKind::MethodDef => MethodDefTable::calc_row_size(self),
			TableKind::ModuleRef => ModuleRefTable::calc_row_size(self),
			TableKind::MethodImpl => MethodImplTable::calc_row_size(self),
			TableKind::FieldLayout => FieldLayoutTable::calc_row_size(self),
			TableKind::ClassLayout => ClassLayoutTable::calc_row_size(self),
			TableKind::PropertyMap => PropertyMapTable::calc_row_size(self),
			TableKind::AssemblyRef => AssemblyRefTable::calc_row_size(self),
			TableKind::FieldMarshal => FieldMarshalTable::calc_row_size(self),
			TableKind::DeclSecurity => DeclSecurityTable::calc_row_size(self),
			TableKind::InterfaceImpl => InterfaceImplTable::calc_row_size(self),
			TableKind::MethodSemantics => MethodSemanticsTable::calc_row_size(self),
			TableKind::CustomAttribute => CustomAttributeTable::calc_row_size(self),
			TableKind::StandAloneSig => StandAloneSignatureTable::calc_row_size(self),
			_ => unimplemented!("Unimplemented table {:?}", table),
		}
	}

	pub(crate) fn idx_size(&self, table: TableKind) -> IndexSize {
		match self.row_count(table) <= u16::MAX as usize {
			true => IndexSize::Slim,
			false => IndexSize::Fat,
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
			write!(f, "heap_sizes: {:b}, ", self.heap_sizes())?;
			write!(f, "valid: {:b}, ", self.valid())?;
			write!(f, "sorted: {:b}, ", self.sorted())?;
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
		fn idx_size(tables: &TableHeap) -> IndexSize;
	}
}
