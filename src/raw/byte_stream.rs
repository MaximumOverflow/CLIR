use std::mem::{align_of, size_of};
use crate::raw::Error::*;
use crate::raw::{MetadataIndex, MetadataIndexSize};

#[derive(Debug, Clone)]
pub struct ByteStream<'l> {
	bytes: &'l [u8],
	position: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Error {
	InvalidData,
	UnalignedRead,
	OffsetOutOfBounds,
	UnexpectedEndOfStream,
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

	pub fn read<T>(&mut self) -> Result<T, Error> {
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
		std::str::from_utf8(bytes).or(Err(InvalidData))
	}

	pub(crate) fn read_index(&mut self, size: MetadataIndexSize) -> Result<MetadataIndex, Error> {
		let value = match size {
			MetadataIndexSize::Slim => self.read::<u16>()? as usize,
			MetadataIndexSize::Fat => self.read::<u32>()? as usize,
		};

		Ok(MetadataIndex(value))
	}
}

pub trait FromByteStream<'l>
where
	Self: Sized,
{
	fn from_byte_stream(stream: &'l mut ByteStream) -> Result<Self, Error>;
}

#[macro_export]
macro_rules! __impl_clone_from_byte_stream {
    ($($T: ty),+) => {
		$(
			impl FromByteStream<'_> for $T {
				fn from_byte_stream(stream: &mut ByteStream) -> Result<Self, Error> {
					stream.read_ref().cloned()
				}
			}
		)+
	};
}
