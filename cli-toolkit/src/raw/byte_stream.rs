pub(crate) use private::*;
pub(crate) use cli_toolkit_derive::FromByteStream;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
	UnalignedRead,
	OffsetOutOfBounds,
	UnexpectedEndOfStream,
	InvalidData(usize, Option<&'static str>),
}

mod private {
	use std::mem::{align_of, size_of};
	use crate::raw::Error::*;
	use crate::raw::{CodedIndex, Error, IndexSize, TableIndex, HeapIndex};

	#[derive(Debug, Clone)]
	pub struct ByteStream<'l> {
		bytes: &'l [u8],
		position: usize,
	}

	pub trait FromByteStream<'l>
	where
		Self: Sized,
	{
		fn from_byte_stream(stream: &'l mut ByteStream) -> Result<Self, Error>;
	}

	impl<'l> ByteStream<'l> {
		pub fn new(bytes: &'l [u8]) -> Self {
			Self { bytes, position: 0 }
		}

		pub fn bytes(&self) -> &'l [u8] {
			self.bytes
		}

		pub fn position(&self) -> usize {
			self.position
		}

		pub fn remaining(&self) -> usize {
			self.bytes.len() - self.position
		}

		pub fn seek(&mut self, position: usize) -> Result<usize, Error> {
			let prev = self.position;
			if position < self.bytes.len() {
				self.position = position;
				Ok(prev)
			} else {
				Err(OffsetOutOfBounds)
			}
		}

		pub fn skip(&mut self, count: usize) -> Result<usize, Error> {
			if self.position + count > self.bytes.len() {
				Err(UnexpectedEndOfStream)
			} else {
				let prev = self.position;
				self.position += count;
				Ok(prev)
			}
		}

		pub fn read<T: 'static>(&mut self) -> Result<T, Error> {
			if self.position + size_of::<T>() > self.bytes.len() {
				return Err(UnexpectedEndOfStream);
			}

			unsafe {
				let ptr = self.bytes.as_ptr().add(self.position);
				let val = std::ptr::read_unaligned(ptr as *const T);
				self.position += size_of::<T>();
				Ok(val)
			}
		}

		pub fn read_checked<T: 'static + PartialEq>(
			&mut self,
			check: impl FnOnce(&T) -> bool,
			message: Option<&'static str>,
		) -> Result<T, Error> {
			let value = self.read::<T>()?;
			match check(&value) {
				true => Ok(value),
				false => Err(InvalidData(self.position - size_of::<T>(), message)),
			}
		}

		pub fn read_ref<T>(&mut self) -> Result<&'l T, Error> {
			if self.position + size_of::<T>() > self.bytes.len() {
				return Err(UnexpectedEndOfStream);
			}

			unsafe {
				let ptr = self.bytes.as_ptr().add(self.position);

				if ptr.align_offset(align_of::<T>()) != 0 {
					return Err(UnalignedRead);
				}

				let val = &*(ptr as *const T);
				self.position += size_of::<T>();
				Ok(val)
			}
		}

		pub fn read_slice<T>(&mut self, count: usize) -> Result<&'l [T], Error> {
			if self.position + size_of::<T>() * count > self.bytes.len() {
				return Err(UnexpectedEndOfStream);
			}

			unsafe {
				let ptr = self.bytes.as_ptr().add(self.position);

				if ptr.align_offset(align_of::<T>()) != 0 {
					return Err(UnalignedRead);
				}

				let val = std::slice::from_raw_parts(ptr as *const T, count);
				self.position += size_of::<T>() * count;
				Ok(val)
			}
		}

		pub fn read_u8_slice_until(&mut self, byte: u8) -> Result<&'l [u8], Error> {
			let start = self.position;
			for b in &self.bytes[start..] {
				self.position += 1;
				if *b == byte {
					return Ok(&self.bytes[start..self.position]);
				}
			}

			Err(UnexpectedEndOfStream)
		}

		pub fn read_null_terminated_str(&mut self) -> Result<&'l str, Error> {
			let bytes = self.read_u8_slice_until(0)?;
			let bytes = &bytes[..bytes.len() - 1];
			std::str::from_utf8(bytes).or(Err(InvalidData(self.position - bytes.len() + 1, None)))
		}

		pub(crate) fn read_table_index(&mut self, size: IndexSize) -> Result<TableIndex, Error> {
			let value = match size {
				IndexSize::Fat => self.read::<u32>()?,
				IndexSize::Slim => self.read::<u16>()? as u32,
			};

			Ok(TableIndex(value))
		}

		pub(crate) fn read_heap_index(&mut self, size: IndexSize) -> Result<HeapIndex, Error> {
			let value = match size {
				IndexSize::Fat => self.read::<u32>()?,
				IndexSize::Slim => self.read::<u16>()? as u32,
			};

			Ok(HeapIndex(value))
		}

		pub(crate) fn read_coded_index(&mut self, size: IndexSize) -> Result<CodedIndex, Error> {
			let value = match size {
				IndexSize::Fat => self.read::<u32>()?,
				IndexSize::Slim => self.read::<u16>()? as u32,
			};

			Ok(CodedIndex(value))
		}
	}
}
