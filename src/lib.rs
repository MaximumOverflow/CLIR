#![allow(unused)]
use crate::metadata::{MetadataIndex, MetadataIndexSize};
use std::mem::{align_of, size_of};
use crate::ParsingError::*;

pub mod tables;
pub mod assembly;
pub mod metadata;
pub mod portable_executable;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ParsingError {
	InvalidUtf8,
	UnalignedRead,
	MissingSection,
	UnexpectedEndOfStream,
}

#[derive(Clone)]
pub struct ZeroCopyReader<'l> {
    bytes: &'l [u8],
    position: usize,
}

impl<'l> ZeroCopyReader<'l> {
    pub fn new(bytes: &'l [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn seek(&mut self, position: usize) -> Result<usize, ParsingError> {
        let prev = self.position;
        if position < self.bytes.len() {
            self.position = position;
            Ok(prev)
        } else {
            Err(UnexpectedEndOfStream)
        }
    }

    pub fn skip(&mut self, count: usize) -> Result<usize, ParsingError> {
        if self.position + count >= self.bytes.len() {
            Err((UnexpectedEndOfStream))
        } else {
            let prev = self.position;
            self.position += count;
            Ok(prev)
        }
    }

    pub fn read<T>(&mut self) -> Result<&'l T, ParsingError> {
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

    pub fn read_slice<T>(&mut self, count: usize) -> Result<&'l [T], ParsingError> {
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

    pub fn read_until(&mut self, byte: u8) -> &'l [u8] {
        let start = self.position;
        for b in &self.bytes[start..] {
            self.position += 1;
            if *b == byte {
                break;
            }
        }

        &self.bytes[start..self.position]
    }

    pub(crate) fn read_index(&mut self, size: MetadataIndexSize) -> Result<MetadataIndex, ParsingError> {
        let value = match size {
            MetadataIndexSize::Slim => *self.read::<u16>()? as usize,
            MetadataIndexSize::Fat => *self.read::<u32>()? as usize,
        };

        Ok(MetadataIndex(value))
    }
}