use std::alloc::{Layout, self};
use std::marker::PhantomData;
use std::mem;
use std::ptr::{NonNull, self};

/// A Cell holds data which can be mutated while being itself immutable.
/// See std::cell::Cell for more info.
#[derive(Clone)]
#[repr(transparent)]
pub struct Cell<T> {
    data: NonNull<T>,
    phantom: PhantomData<T>
}
impl<T> Cell<T> {
    /// Creates a new cell which contains the given value.
    pub fn new(value: T) -> Cell<T> {
        unsafe {
            let align = mem::align_of::<T>();
            let size = mem::size_of::<T>();
            let ptr = alloc::alloc(Layout::from_size_align(size, align)
                                      .expect("Error allocating memory"));
            let nonnull = NonNull::new(ptr as *mut T).unwrap();
            *nonnull.as_ptr() = value;
            Cell { data: nonnull, phantom: PhantomData }
        }
    }

    /// Returns a reference to the item in the cell
    pub fn as_ref(&self) -> &T {
        unsafe {
            &(*self.data.as_ptr())
        }
    }

    /// Returns a mutable reference to the item in the cell
    pub fn as_mut(&self) -> &mut T {
        unsafe {
            &mut(*self.data.as_ptr())
        }
    }

    /// Takes the value out of this Cell and returns it, while
    /// destroying the Cell
    pub fn take(self) -> T {
        unsafe {
            ptr::read(self.data.as_ptr())
        }
    }
}

impl<T> Drop for Cell<T> {
    // We need to deallocate our pointers
    fn drop(&mut self) {
        unsafe {
            let align = mem::align_of::<T>();
            let size = mem::size_of::<T>();
            alloc::dealloc(self.data.as_ptr() as *mut u8,
                        Layout::from_size_align(
                            size,
                            align
                        ).expect("Unexpected panic in drop"));
        }
    }
}
