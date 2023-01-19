use crate::metadata::private::IsMetadataHeap;
use crate::tables::{AssemblyTable, DummyTable, GenericMetadataTable, MetadataTable, ModuleTable, Module, TableKind, TableKindIter, TypeRef, TypeRefTable};
use crate::{ParsingError, ZeroCopyReader};
use indoc::indoc;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use strum::IntoEnumIterator;
use crate::ParsingError::InvalidUtf8;

#[derive(Debug)]
pub struct MetadataHeader<'l> {
    offset: usize,
    pub signature: u32,
    pub major_version: u16,
    pub minor_version: u16,
    #[allow(unused)]
    reserved: u32,
    pub length: u32,
    pub version: &'l str,
    pub flags: u16,
    pub streams: u16,
    stream_headers: &'l [u8],
}

impl<'l> MetadataHeader<'l> {
    pub(crate) fn new(bytes: &'l [u8], position: usize) -> Result<Self, ParsingError> {
        let mut reader = ZeroCopyReader::new(bytes);
        reader.seek(position)?;

        let signature = reader.read::<u32>()?.clone();
        let major_version = reader.read::<u16>()?.clone();
        let minor_version = reader.read::<u16>()?.clone();
        let reserved = reader.read::<u32>()?.clone();
        let length = reader.read::<u32>()?.clone();
        let version = {
            let bytes = reader.read_until(0);
            let version = std::str::from_utf8(&bytes[..bytes.len() - 1]).map_err(|_| InvalidUtf8)?;
            let skip = length as usize - version.len() - 1;
            reader.skip(skip)?;
            version
        };
        let flags = reader.read::<u16>()?.clone();
        let streams = reader.read::<u16>()?.clone();
        let stream_headers = {
            let start = reader.position();
            for _ in 0..streams {
                reader.skip(8)?;
                let name = reader.read_until(0);
                reader.skip((4usize.wrapping_sub(name.len())) % 4)?;
            }

            &bytes[start..reader.position()]
        };

        Ok(Self {
            offset: position,
            signature,
            major_version,
            minor_version,
            reserved,
            length,
            version,
            flags,
            streams,
            stream_headers,
        })
    }

    pub fn stream_headers(&self) -> StreamHeaderIterator {
        StreamHeaderIterator {
            reader: ZeroCopyReader::new(self.stream_headers),
        }
    }

    pub(crate) fn get_heaps<'a>(
        &self,
        assembly_bytes: &'a [u8],
    ) -> HashMap<TypeId, Box<dyn MetadataHeap<'a> + 'a>> {
        let mut streams = HashMap::<TypeId, Box<dyn MetadataHeap + 'a>>::new();

        for header in self.stream_headers() {
            let offset = self.offset + header.offset as usize;

            match header.name {
                "#~" => streams.insert(
                    TypeId::of::<TableHeap>(),
                    Box::new(TableHeap {
                        bytes: &assembly_bytes[offset..offset + header.size as usize],
                    }),
                ),

                "#Strings" => streams.insert(
                    TypeId::of::<StringHeap>(),
                    Box::new(StringHeap {
                        bytes: &assembly_bytes[offset..offset + header.size as usize],
                    }),
                ),

                "#US" => streams.insert(
                    TypeId::of::<UserStringHeap>(),
                    Box::new(UserStringHeap {
                        bytes: &assembly_bytes[offset..offset + header.size as usize],
                    }),
                ),

                "#GUID" => streams.insert(
                    TypeId::of::<GuidHeap>(),
                    Box::new(GuidHeap {
                        bytes: &assembly_bytes[offset..offset + header.size as usize],
                    }),
                ),

                unknown => unreachable!("Unknown metadata heap {}", unknown),
            };
        }

        streams
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct StreamHeader<'l> {
    offset: u32,
    size: u32,
    name: &'l str,
}

pub struct StreamHeaderIterator<'l> {
    reader: ZeroCopyReader<'l>,
}

impl<'l> Iterator for StreamHeaderIterator<'l> {
    type Item = StreamHeader<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.reader.read::<u32>().ok()?.clone();
        let size = self.reader.read::<u32>().ok()?.clone();
        let name = self.reader.read_until(0);
        self.reader
            .skip((4usize.wrapping_sub(name.len())) % 4)
            .ok()?;
        let name = std::str::from_utf8(&name[..name.len() - 1]).ok()?;
        Some(StreamHeader { offset, size, name })
    }
}

pub struct StringHeap<'l> {
    bytes: &'l [u8],
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
        write!(f, "{:?}", std::str::from_utf8(self.bytes).unwrap())
    }
}

#[derive(Clone)]
pub struct TableHeap<'l> {
    bytes: &'l [u8],
}

impl<'l> TableHeap<'l> {
    pub fn major_version(&self) -> u8 {
        self.bytes[4]
    }

    pub fn minor_version(&self) -> u8 {
        self.bytes[5]
    }

    pub fn heap_sizes(&self) -> u8 {
        self.bytes[6]
    }

    pub fn valid(&self) -> u64 {
        let mut valid = [0; 8];
        valid.copy_from_slice(&self.bytes[8..16]);
        u64::from_le_bytes(valid)
    }

    pub fn sorted(&self) -> u64 {
        let mut valid = [0; 8];
        valid.copy_from_slice(&self.bytes[16..24]);
        u64::from_le_bytes(valid)
    }

    pub fn table_count(&self) -> usize {
        self.valid().count_ones() as usize
    }

    pub fn has_table(&self, kind: TableKind) -> bool {
        (self.valid() & (1 << kind as u64)) != 0
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
		let mut iter = TableKind::iter();
		while let Some(kind) = iter.next() {
			if kind == table { break }
			else { index += self.has_table(kind) as usize; }
		}
		
		return self.rows()[index] as usize;
	}

    pub fn tables(&self) -> TableIterator {
        TableIterator {
            heap: self.clone(),
            reader: ZeroCopyReader {
                bytes: self.bytes,
                position: 24 + 4 * self.table_count(),
            },
            iter: TableKind::iter(),
        }
    }

    pub fn heap_idx_size<T: for<'a> MetadataHeap<'a> + 'static>(&self) -> MetadataIndexSize {
        let flags = self.heap_sizes();

        let size = if TypeId::of::<T>() == TypeId::of::<StringHeap>() {
            2 + ((flags & 0x1) != 0) as usize
        } else if TypeId::of::<T>() == TypeId::of::<GuidHeap>() {
            2 + ((flags & 0x2) != 0) as usize
        } else if TypeId::of::<T>() == TypeId::of::<BlobHeap>() {
            2 + ((flags & 0x4) != 0) as usize
        } else {
            2
        };

        match size {
            2 => MetadataIndexSize::Slim,
            4 => MetadataIndexSize::Fat,
            _ => unreachable!(),
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
						heap_sizes:    0b{:08b},
						valid:         0b{:064b},
						sorted:        0b{:064b},
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

pub struct TableIterator<'l> {
    heap: TableHeap<'l>,
    iter: TableKindIter,
    reader: ZeroCopyReader<'l>,
}

impl<'l> Iterator for TableIterator<'l> {
    type Item = GenericMetadataTable<'l>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let kind = self.iter.next()?;
            if !self.heap.has_table(kind) {
                continue;
            }

			let rows = self.heap.row_count(kind);
			
            let table = match kind {
                TableKind::Module => {
                    let guid_idx_size = self.heap.heap_idx_size::<GuidHeap>();
                    let str_idx_size = self.heap.heap_idx_size::<StringHeap>();
                    let row_size = Module::row_size(guid_idx_size, str_idx_size);

                    ModuleTable::new(
						self.reader.read_slice::<u8>(rows * row_size).ok()?,
						guid_idx_size,
						str_idx_size,
                    )
                    .into()
                }
				
				TableKind::TypeRef => {
					let str_idx_size = self.heap.heap_idx_size::<StringHeap>();
					let res_idx_size = get_coded_index_size(CodedIndexKind::TypeOrMethodDef, &self.heap);
					let row_size = TypeRef::row_size(res_idx_size, str_idx_size);
					
					TypeRefTable::new(
						self.reader.read_slice::<u8>(rows * row_size).ok()?,
						res_idx_size,
						str_idx_size
					).into()
				}
				
				TableKind::Assembly => {
					let blob_idx_size = self.heap.heap_idx_size::<BlobHeap>();
					let string_idx_size = self.heap.heap_idx_size::<StringHeap>();
					let assembly_table_row_size = 16 + blob_idx_size as usize + string_idx_size as usize * 2;
					
					AssemblyTable::new(
						self.reader.read_slice::<u8>(rows * assembly_table_row_size).ok()?,
						blob_idx_size,
						string_idx_size
					).ok()?.into()
				}

                _ => unimplemented!("Unimplemented table kind {:?}", kind),
            };

            return Some(table);
        }
    }
}

#[derive(Debug)]
pub struct GuidHeap<'l> {
    bytes: &'l [u8],
}

#[derive(Debug)]
pub struct BlobHeap<'l> {
    bytes: &'l [u8],
}

pub struct UserStringHeap<'l> {
    bytes: &'l [u8],
}

impl Debug for UserStringHeap<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", std::str::from_utf8(self.bytes).unwrap())
    }
}

pub trait MetadataHeap<'l>
where
    Self: IsMetadataHeap + Debug,
{
}

impl<T: IsMetadataHeap + Debug> MetadataHeap<'_> for T {}

mod private {
    use crate::metadata::*;
    pub trait IsMetadataHeap {}
    impl IsMetadataHeap for BlobHeap<'_> {}
    impl IsMetadataHeap for GuidHeap<'_> {}
    impl IsMetadataHeap for TableHeap<'_> {}
    impl IsMetadataHeap for StringHeap<'_> {}
    impl IsMetadataHeap for UserStringHeap<'_> {}
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
pub struct MetadataIndex(pub(crate) usize);

#[derive(Debug, Copy, Clone)]
pub enum MetadataIndexSize {
    Slim = 0x2,
    Fat = 0x4,
}

pub enum CodedIndexKind {
    TypeDefOrRef,
    HasConstant,
    HasCustomAttribute,
    HasFieldMarshal,
    HasDeclSecurity,
    MemberRefParent,
    HasSemantics,
    MethodDefOrRef,
    MemberForwarded,
    Implementation,
    CustomAttributeType,
    ResolutionScope,
    TypeOrMethodDef,
    HasCustomDebugInformation,
}

fn get_coded_index_size(kind: CodedIndexKind, tables_heap: &TableHeap) -> MetadataIndexSize {
    let (bits, tables): (usize, &[TableKind]) = match kind {
        CodedIndexKind::TypeDefOrRef => (
            2,
            &[TableKind::TypeDef, TableKind::TypeRef, TableKind::TypeSpec],
        ),
        CodedIndexKind::HasConstant => (
            2,
            &[TableKind::Field, TableKind::Param, TableKind::Property],
        ),
        CodedIndexKind::HasCustomAttribute => (
            5,
            &[
                TableKind::Method,
                TableKind::Field,
                TableKind::TypeRef,
                TableKind::TypeDef,
                TableKind::Param,
                TableKind::InterfaceImpl,
                TableKind::MemberRef,
                TableKind::Module,
                TableKind::DeclSecurity,
                TableKind::Property,
                TableKind::Event,
                TableKind::StandAloneSig,
                TableKind::ModuleRef,
                TableKind::TypeSpec,
                TableKind::Assembly,
                TableKind::AssemblyRef,
                TableKind::File,
                TableKind::ExportedType,
                TableKind::ManifestResource,
                TableKind::GenericParam,
                TableKind::GenericParamConstraint,
                TableKind::MethodSpec,
            ],
        ),
        CodedIndexKind::HasFieldMarshal => (1, &[TableKind::Field, TableKind::Param]),
        CodedIndexKind::HasDeclSecurity => (
            2,
            &[TableKind::TypeDef, TableKind::Method, TableKind::Assembly],
        ),
        CodedIndexKind::MemberRefParent => (
            3,
            &[
                TableKind::TypeDef,
                TableKind::TypeRef,
                TableKind::ModuleRef,
                TableKind::Method,
                TableKind::TypeSpec,
            ],
        ),
        CodedIndexKind::HasSemantics => (1, &[TableKind::Event, TableKind::Property]),
        CodedIndexKind::MethodDefOrRef => (1, &[TableKind::Method, TableKind::MemberRef]),
        CodedIndexKind::MemberForwarded => (1, &[TableKind::Field, TableKind::Method]),
        CodedIndexKind::Implementation => (
            2,
            &[
                TableKind::File,
                TableKind::AssemblyRef,
                TableKind::ExportedType,
            ],
        ),
        CodedIndexKind::CustomAttributeType => (3, &[TableKind::Method, TableKind::MemberRef]),
        CodedIndexKind::ResolutionScope => (
            2,
            &[
                TableKind::Module,
                TableKind::ModuleRef,
                TableKind::AssemblyRef,
                TableKind::TypeRef,
            ],
        ),
        CodedIndexKind::TypeOrMethodDef => (1, &[TableKind::TypeDef, TableKind::Method]),
        CodedIndexKind::HasCustomDebugInformation => (
            5,
            &[
                TableKind::Method,
                TableKind::Field,
                TableKind::TypeRef,
                TableKind::TypeDef,
                TableKind::Param,
                TableKind::InterfaceImpl,
                TableKind::MemberRef,
                TableKind::Module,
                TableKind::DeclSecurity,
                TableKind::Property,
                TableKind::Event,
                TableKind::StandAloneSig,
                TableKind::ModuleRef,
                TableKind::TypeSpec,
                TableKind::Assembly,
                TableKind::AssemblyRef,
                TableKind::File,
                TableKind::ExportedType,
                TableKind::ManifestResource,
                TableKind::GenericParam,
                TableKind::GenericParamConstraint,
                TableKind::MethodSpec,
                TableKind::Document,
                TableKind::LocalScope,
                TableKind::LocalVariable,
                TableKind::LocalConstant,
                TableKind::ImportScope,
            ],
        ),
    };
	
	let map = |t: &TableKind| tables_heap.row_count(*t);
	match tables.iter().map(map).max().unwrap() < (1 << (16 - bits)) {
		true => MetadataIndexSize::Slim,
		false => MetadataIndexSize::Fat,
	}
}
