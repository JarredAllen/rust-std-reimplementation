#![feature(ptr_internals)]

use std::alloc::{Layout, self};
use std::mem;
use std::ptr::{NonNull, self};

/// A re-implementation of the Vec class in the rust std. This is done
/// purely for pedagogigal value, and is not something worth actually
/// using.
pub struct Vec<T> {
    pointer: NonNull<T>,
    capacity: usize,
    length: usize,
}

impl<T> Vec<T> {
    /// Create a new empty Vec
    pub fn new() -> Self {
        assert!(mem::size_of::<T>() != 0, "Zero-length types not yet implemented");
        Vec { pointer: NonNull::dangling(), capacity: 0, length: 0 }
    }

    /// Resize the Vec. If it has no space allocated, it allocates space
    /// for one element. If it has space allocated, it doubles the
    /// amount of allocated space.
    fn grow(&mut self) {
        unsafe {
            // Need to manually specify the alignment and size allocated
            let align = mem::align_of::<T>();
            let elem_size = mem::size_of::<T>();

            let (new_cap, ptr) = if self.capacity == 0 {
                // The array was empty, so we make a new array of size 1
                let ptr = alloc::alloc(Layout::from_size_align(elem_size, align)
                                          .expect("Error allocating Vec"));
                (1, ptr)
            } else {
                // Make a new array, and then copy it over
                let new_cap = self.capacity * 2;
                let old_num_bytes = self.capacity * elem_size;

                // LLVM's GEP behaves poorly if you use an index greater
                // than the max value in an isize.
                // To accomplish this on a 64-bit architecture without
                // ZSTs would require >8EB of memory (unlikely), or more
                // if your type is > 1 byte in size, but this is
                // preserved for 32-bit machines.
                assert!(old_num_bytes <= (::std::isize::MAX as usize) / 2, "too many things");

                let new_num_bytes = old_num_bytes * 2;
                // Here we actually reallocate the array
                let ptr = alloc::realloc(self.pointer.as_ptr() as *mut u8,
                                            Layout::from_size_align(
                                                old_num_bytes,
                                                align
                                            ).expect("Error re-allocating Vec"),
                                            new_num_bytes);
                (new_cap, ptr)
            };

            // If the expect is hit, then we somehow ran out of memory.
            // Given that the OS can use paging and will likely shut us
            // down before we get to ridiculous amounts of memory, this
            // probably means we requested far more space than exists in
            // one go.
            self.pointer = NonNull::new(ptr as *mut T).expect("Out of memory in Vec reallocate");
            self.capacity = new_cap;
        }
    }

    /// Append a value to the end of the Vec, reallocating if more space
    /// is necessary.
    /// Guaranteed to run in O(n) time, O(1) amortized
    pub fn push(&mut self, element: T) {
        if self.length == self.capacity {
            self.grow();
        }
        unsafe {
            ptr::write(self.pointer.as_ptr().add(self.length), element);
        }
        self.length += 1;
    }

    /// Removes the last item from the Vec and returns it
    pub fn pop(&mut self) -> Option<T> {
        if self.length == 0 {
            None
        } else {
            self.length -= 1;
            unsafe {
                Some(ptr::read(self.pointer.as_ptr().add(self.length)))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_push_pop() {
        let mut v: Vec<i64> = Vec::new();
        v.push(1);
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(5);
        assert_eq!(v.pop(), Some(5));
        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.pop(), Some(2));
        v.push(17);
        assert_eq!(v.pop(), Some(17));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), None);
    }
}
