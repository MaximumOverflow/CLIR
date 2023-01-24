use std::alloc::{Layout, LayoutError};
use std::fmt::{Debug, Formatter, Pointer};
use std::ops::{Deref, Index};
use std::rc::Rc;

pub struct IndexedRcRef<T, C: Index<usize, Output = T> + ?Sized> {
	index: usize,
	container: Rc<C>,
}

impl<T, C: Index<usize, Output = T> + ?Sized> IndexedRcRef<T, C> {
	pub fn new(container: Rc<C>, index: usize) -> Self {
		Self { container, index }
	}
}

impl<T, C: Index<usize, Output = T> + ?Sized> Deref for IndexedRcRef<T, C> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.container[self.index]
	}
}

impl<T: Debug, C: Index<usize, Output = T> + ?Sized> Debug for IndexedRcRef<T, C> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		Debug::fmt(self.deref(), f)
	}
}

pub(crate) unsafe fn get_mut_unchecked<'l, T: ?Sized>(rc: &Rc<T>) -> &mut T {
	&mut *(Rc::as_ptr(&rc) as *mut T)
}
