use std::alloc::{Layout, self};
use std::mem;
use std::ops::{Deref, DerefMut};
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

    /// Returns the number of items in the Vec
    pub fn length(&mut self) -> usize {
        return self.length;
    }

    /// Inserts the element into the given index and pushes the rest of
    /// the elements after it back.
    pub fn insert(&mut self, index: usize, element: T) {
        assert!(index <= self.length, "index out of bounds");

        if self.length == self.capacity {
            self.grow();
        }

        unsafe {
            if index < self.length {
                ptr::copy(
                    self.pointer.as_ptr().add(index),
                    self.pointer.as_ptr().add(index+1),
                    self.length - index);
            }
            ptr::write(self.pointer.as_ptr().add(index), element);
        }
        self.length += 1;
    }

    /// Removes and returns the element at the index, and shifts
    /// everything after it forward one index
    pub fn remove(&mut self, index: usize) -> T {
        assert!(index < self.length, "index out of bounds error");
        unsafe {
            self.length -= 1;
            let result = ptr::read(self.pointer.as_ptr().add(index));
            ptr::copy(self.pointer.as_ptr().add(index + 1),
                      self.pointer.as_ptr().add(index),
                      self.length - index);
            result
        }
    }

    /// Consumes this Vec object and creates an IntoIter which iterates
    /// over the elements of this Vec
    pub fn into_iter(self) -> IntoIter<T> {
        let pointer = self.pointer;
        let capacity = self.capacity;
        let length = self.length;

        mem::forget(self);
        unsafe {
            IntoIter {
                buffer: pointer,
                capacity,
                start: pointer.as_ptr(),
                end: if capacity == 0 {
                    pointer.as_ptr()
                } else {
                    pointer.as_ptr().add(length)
                }
            }
        }
    }
}

impl<T> Drop for Vec<T> {
    /// Drop all elements in the Vec and then deallocate resources,
    /// because T may need to be dropped.
    fn drop(&mut self) {
        if self.capacity != 0 {
            while let Some(_) = self.pop() {}
            let align = mem::align_of::<T>();
            let num_bytes = mem::size_of::<T>() * self.capacity;
            unsafe {
                alloc::dealloc(self.pointer.as_ptr() as *mut u8,
                                Layout::from_size_align(
                                    num_bytes,
                                    align
                                ).expect("Unexpected panic while deallocating"));
            }
        }
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(self.pointer.as_ptr(), self.length)
        }
    }
}
impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.pointer.as_ptr(), self.length)
        }
    }
}

pub struct IntoIter<T> {
    buffer: NonNull<T>,
    capacity: usize,
    start: *const T,
    end: *const T,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let result = ptr::read(self.start);
                self.start = self.start.add(1);
                Some(result)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let length = (self.end as usize - self.start as usize) / mem::size_of::<T>();
        (length, Some(length))
    }
}
impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                self.end = self.end.sub(1);
                Some(ptr::read(self.end))
            }
        }
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if self.capacity != 0 {
            let align = mem::align_of::<T>();
            let num_bytes = mem::size_of::<T>() * self.capacity;
            unsafe {
                while self.start != self.end {
                    self.next();
                }
                alloc::dealloc(self.buffer.as_ptr() as *mut u8,
                                Layout::from_size_align(
                                    num_bytes,
                                    align
                                ).expect("Unexpected panic while deallocating"));
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Vec;

    #[test]
    pub fn test_push_pop() {
        let mut v: Vec<i64> = Vec::new();
        v.push(1);
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(5);
        assert_eq!(v.length(), 5);
        assert_eq!(v.pop(), Some(5));
        assert_eq!(v.pop(), Some(3));
        assert_eq!(v.pop(), Some(2));
        v.push(17);
        assert_eq!(v.length(), 3);
        assert_eq!(v.pop(), Some(17));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), Some(1));
        assert_eq!(v.pop(), None);
    }

    #[test]
    pub fn test_slice() {
        let mut v: Vec<i64> = Vec::new();
        v.push(1);
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(5);
        v.push(7);
        v.push(13);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 1);
        assert_eq!(v[6], 13);
        assert_eq!(v[5], 7);
        v[5] = 8;
        assert_eq!(v[5], 8);
    }

    #[test]
    pub fn test_insert() {
        let mut v: Vec<i64> = Vec::new();
        v.push(1);
        v.push(1);
        v.push(2);
        v.push(3);
        v.push(5);
        v.push(13);
        v.insert(5, 8);
        v.insert(7, 21);
        v.insert(0, 0);
        assert_eq!(v.length(), 9);
        assert_eq!(v[0], 0);
        assert_eq!(v[2], 1);
        assert_eq!(v[6], 8);
        assert_eq!(v[7], 13);
        assert_eq!(v[8], 21);
    }

    #[test]
    pub fn test_remove() {
        let mut v: Vec<i64> = Vec::new();
        v.push(7);
        v.push(1);
        v.push(1);
        v.push(2);
        v.push(7);
        v.push(3);
        v.push(5);
        v.push(8);
        v.push(13);
        v.push(7);
        v.remove(9);
        v.remove(4);
        v.remove(0);
        assert_eq!(v.length(), 7);
        assert_eq!(v[0], 1);
        assert_eq!(v[1], 1);
        assert_eq!(v[3], 3);
        assert_eq!(v[6], 13);
    }

    #[test]
    pub fn test_into_iter() {
        let mut vec: Vec<i64> = Vec::new();
        vec.push(1);
        vec.push(1);
        vec.push(2);
        vec.push(3);
        vec.push(5);
        vec.push(8);
        vec.push(13);
        let mut iter = vec.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(13));
        assert_eq!(iter.next_back(), Some(8));
        assert_eq!(iter.next_back(), Some(5));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }
}
