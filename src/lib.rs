use std::{
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    mem::swap,
    ops::{Index, IndexMut},
    ptr::null_mut,
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const INDEX_BOUNDS_ERROR: &str = "Index is out of bounds";

/// performs XOR on 2 pointers and returns the resulting pointer
#[inline]
fn xor_ptrs<T>(first_ptr: *mut XorNode<T>, second_ptr: *mut XorNode<T>) -> *mut XorNode<T> {
    let first_ptr_value = first_ptr as usize;
    let second_ptr_value = second_ptr as usize;
    let new_ptr_value = first_ptr_value ^ second_ptr_value;
    new_ptr_value as _
}

struct XorNode<T> {
    payload: T,
    xor_ptr: *mut XorNode<T>,
}
impl<T> XorNode<T> {
    fn allocate(value: T) -> *mut Self {
        Box::leak(Box::new(Self {
            payload: value,
            xor_ptr: null_mut(),
        }))
    }
}

/// linked list using single XOR pointer nodes
pub struct XorLinkedList<T> {
    size: usize,
    start: *mut XorNode<T>,
    end: *mut XorNode<T>,
}
impl<T> XorLinkedList<T> {
    /// creates an empty XOR linked list
    pub fn new() -> Self {
        Self {
            size: 0,
            start: null_mut(),
            end: null_mut(),
        }
    }

    /// removes all elements from the list
    pub fn clear(&mut self) {
        loop {
            if self.pop_front().is_none() {
                return;
            }
        }
    }

    /// returns a reference of the first element if present
    pub fn peek_front(&self) -> Option<&T> {
        if self.size == 0 {
            debug_assert!(self.start.is_null());
            debug_assert!(self.end.is_null());
            None
        } else {
            unsafe { Some(&(*self.start).payload) }
        }
    }

    /// returns a mutable of reference the first element if present
    pub fn peek_front_mut(&mut self) -> Option<&mut T> {
        if self.size == 0 {
            debug_assert!(self.start.is_null());
            debug_assert!(self.end.is_null());
            None
        } else {
            unsafe { Some(&mut (*self.start).payload) }
        }
    }

    /// returns a reference of the last element if present
    pub fn peek_back(&self) -> Option<&T> {
        if self.size == 0 {
            debug_assert!(self.start.is_null());
            debug_assert!(self.end.is_null());
            None
        } else {
            unsafe { Some(&(*self.end).payload) }
        }
    }

    /// returns a mutable reference of the last element if present
    pub fn peek_back_mut(&mut self) -> Option<&mut T> {
        if self.size == 0 {
            debug_assert!(self.start.is_null());
            debug_assert!(self.end.is_null());
            None
        } else {
            unsafe { Some(&mut (*self.end).payload) }
        }
    }

    /// returns a reference the element at the index
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.size {
            return None;
        }

        let ptr = unsafe { self.get_ptr_at(index) };

        Some(unsafe { &(*ptr).payload })
    }

    /// returns a mutable reference the element at the index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.size {
            return None;
        }

        let ptr = unsafe { self.get_ptr_at(index) };

        Some(unsafe { &mut (*ptr).payload })
    }

    #[inline]
    unsafe fn get_ptr_at(&self, index: usize) -> *mut XorNode<T> {
        debug_assert!(index < self.size);
        let mut prev_ptr = null_mut();
        let (mut current_ptr, mut jump_count) = if index > self.size / 2 {
            (self.end, self.size - index - 1)
        } else {
            (self.start, index)
        };
        while jump_count > 0 {
            let new_ptr;
            unsafe {
                new_ptr = xor_ptrs((*current_ptr).xor_ptr, prev_ptr);
            }
            prev_ptr = current_ptr;
            current_ptr = new_ptr;
            jump_count -= 1;
        }

        current_ptr
    }

    /// returns the number of elements
    pub fn len(&self) -> usize {
        self.size
    }

    /// returns true if the list is empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    unsafe fn push_end(end_ptr1: &mut *mut XorNode<T>, end_ptr2: &mut *mut XorNode<T>, value: T) {
        let new_node = XorNode::allocate(value);

        if end_ptr2.is_null() {
            debug_assert!(end_ptr1.is_null());
            *end_ptr1 = new_node;
            *end_ptr2 = new_node;
        } else {
            unsafe {
                (**end_ptr2).xor_ptr = xor_ptrs((**end_ptr2).xor_ptr, new_node);
                (*new_node).xor_ptr = *end_ptr2;
            }
            *end_ptr2 = new_node
        }
    }

    /// inserts an element to the end of the list
    pub fn push_back(&mut self, value: T) {
        self.size += 1;
        unsafe {
            Self::push_end(&mut self.start, &mut self.end, value);
        }
    }

    /// inserts an element to the start of the list
    pub fn push_front(&mut self, value: T) {
        self.size += 1;
        unsafe {
            Self::push_end(&mut self.end, &mut self.start, value);
        }
    }

    #[inline]
    unsafe fn pop_end(
        size: &mut usize,
        end_ptr1: &mut *mut XorNode<T>,
        end_ptr2: &mut *mut XorNode<T>,
    ) -> Option<T> {
        if end_ptr1.is_null() {
            debug_assert!(end_ptr2.is_null());
            debug_assert_eq!(0, *size);
            return None;
        }
        let old_ptr = *end_ptr1;
        unsafe {
            if end_ptr1 == end_ptr2 {
                *end_ptr1 = null_mut();
                *end_ptr2 = null_mut();
            } else {
                *end_ptr1 = (**end_ptr1).xor_ptr;
                (**end_ptr1).xor_ptr = xor_ptrs((**end_ptr1).xor_ptr, old_ptr);
            }

            *size -= 1;
            Some(Box::from_raw(old_ptr).payload)
        }
    }

    /// removes and returns the element from the start of the list
    pub fn pop_front(&mut self) -> Option<T> {
        unsafe { Self::pop_end(&mut self.size, &mut self.start, &mut self.end) }
    }

    /// removes and returns the element from the end of the list
    pub fn pop_back(&mut self) -> Option<T> {
        unsafe { Self::pop_end(&mut self.size, &mut self.end, &mut self.start) }
    }

    /// returns an iterator of element references from the start to the end of the list
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.into_iter()
    }

    /// returns an iterator of mutable element references from the start to the end of the list
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.into_iter()
    }

    /// returns an iterator from the end to the start of the list
    pub fn into_reverse_iter(self) -> impl Iterator<Item = T> {
        ReverseXorLinkedListIter {
            xor_linked_list: self,
        }
    }

    /// returns an iterator of element references from the end to the start of the list
    pub fn reverse_iter(&self) -> impl Iterator<Item = &T> {
        RefXorLinkedListIter {
            xor_linked_list_lifetime: PhantomData,
            current_ptr: self.end,
            prev_ptr: null_mut(),
        }
    }

    /// returns an iterator of mutable element references from the end to the start of the list
    pub fn reverse_iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let current_ptr = self.end;
        MutRefXorLinkedListIter {
            xor_linked_list_lifetime: PhantomData,
            current_ptr,
            prev_ptr: null_mut(),
        }
    }

    /// reverses the order of the list
    pub fn reverse(&mut self) {
        swap(&mut self.start, &mut self.end);
    }

    /// returns a tuple of the pointers at index and index-1, where 0 < index < size-1
    #[inline]
    unsafe fn get_ptr_at_and_prev(&mut self, index: usize) -> (*mut XorNode<T>, *mut XorNode<T>) {
        let mut prev_ptr = null_mut();
        let is_backwards_iteration = index > self.size / 2;
        let (mut current_ptr, mut jump_count) = if is_backwards_iteration {
            (self.end, self.size - index)
        } else {
            (self.start, index)
        };
        while jump_count > 0 {
            let new_ptr;
            unsafe {
                new_ptr = xor_ptrs((*current_ptr).xor_ptr, prev_ptr);
            }
            prev_ptr = current_ptr;
            current_ptr = new_ptr;
            jump_count -= 1;
        }
        // swap current and previous for backwards iteration (previous is after current)
        if is_backwards_iteration {
            swap(&mut current_ptr, &mut prev_ptr);
        }

        (current_ptr, prev_ptr)
    }

    /// inserts an element at the index
    pub fn insert_at(&mut self, index: usize, value: T) {
        assert!(
            index <= self.size,
            "Index is greater than the size {}",
            self.size
        );
        if index == 0 {
            self.push_front(value);
        } else if index == self.size {
            self.push_back(value);
        } else {
            unsafe {
                let (current_ptr, prev_ptr) = self.get_ptr_at_and_prev(index);
                (*current_ptr).xor_ptr = xor_ptrs((*current_ptr).xor_ptr, prev_ptr);
                (*prev_ptr).xor_ptr = xor_ptrs((*prev_ptr).xor_ptr, current_ptr);

                let new_node = XorNode::allocate(value);
                (*new_node).xor_ptr = xor_ptrs(current_ptr, prev_ptr);

                (*current_ptr).xor_ptr = xor_ptrs((*current_ptr).xor_ptr, new_node);
                (*prev_ptr).xor_ptr = xor_ptrs((*prev_ptr).xor_ptr, new_node);
            }
            self.size += 1;
        }
    }

    /// removes and returns the value at the index
    pub fn remove_at(&mut self, index: usize) -> Option<T> {
        if index >= self.size {
            None
        } else if index == 0 {
            self.pop_front()
        } else if index + 1 == self.size {
            self.pop_back()
        } else {
            unsafe {
                let (current_ptr, prev_ptr) = self.get_ptr_at_and_prev(index);
                let next_ptr = xor_ptrs((*current_ptr).xor_ptr, prev_ptr);
                (*next_ptr).xor_ptr = xor_ptrs((*next_ptr).xor_ptr, current_ptr);
                (*prev_ptr).xor_ptr = xor_ptrs((*prev_ptr).xor_ptr, current_ptr);

                (*next_ptr).xor_ptr = xor_ptrs((*next_ptr).xor_ptr, prev_ptr);
                (*prev_ptr).xor_ptr = xor_ptrs((*prev_ptr).xor_ptr, next_ptr);
                self.size -= 1;

                Some(Box::from_raw(current_ptr).payload)
            }
        }
    }
}
impl<T: PartialEq> PartialEq for XorLinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().zip(other.iter()).all(|(a, b)| a == b)
    }
}
impl<T: Eq> Eq for XorLinkedList<T> {}
impl<T: Hash> Hash for XorLinkedList<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for element in self {
            element.hash(state);
        }
    }
}
impl<T> Extend<T> for XorLinkedList<T> {
    fn extend<A: IntoIterator<Item = T>>(&mut self, iter: A) {
        for element in iter {
            self.push_back(element);
        }
    }
}
impl<T> Index<usize> for XorLinkedList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect(INDEX_BOUNDS_ERROR)
    }
}
impl<T> IndexMut<usize> for XorLinkedList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect(INDEX_BOUNDS_ERROR)
    }
}
impl<T: Clone> Clone for XorLinkedList<T> {
    fn clone(&self) -> Self {
        let mut cloned_list = XorLinkedList::new();
        for element in self {
            cloned_list.push_back(element.clone());
        }
        cloned_list
    }
}
impl<T> Default for XorLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T: Debug> Debug for XorLinkedList<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&*self).finish()
    }
}
impl<T> Drop for XorLinkedList<T> {
    fn drop(&mut self) {
        self.clear();
    }
}
impl<T> IntoIterator for XorLinkedList<T> {
    type Item = T;

    type IntoIter = XorLinkedListIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            xor_linked_list: self,
        }
    }
}
impl<'a, T> IntoIterator for &'a XorLinkedList<T> {
    type Item = &'a T;
    type IntoIter = RefXorLinkedListIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let current_ptr = self.start;
        let prev_ptr = null_mut();

        RefXorLinkedListIter {
            xor_linked_list_lifetime: PhantomData,
            current_ptr,
            prev_ptr,
        }
    }
}
impl<'a, T> IntoIterator for &'a mut XorLinkedList<T> {
    type Item = &'a mut T;
    type IntoIter = MutRefXorLinkedListIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        let current_ptr = self.start;
        let prev_ptr = null_mut();

        MutRefXorLinkedListIter {
            xor_linked_list_lifetime: PhantomData,
            current_ptr,
            prev_ptr,
        }
    }
}
impl<T> FromIterator<T> for XorLinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = XorLinkedList::new();
        for element in iter {
            list.push_back(element);
        }

        list
    }
}
#[cfg(feature = "serde")]
impl<T: Serialize> Serialize for XorLinkedList<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.iter())
    }
}
#[cfg(feature = "serde")]
impl<'de, T: Deserialize<'de>> Deserialize<'de> for XorLinkedList<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let vec = Vec::<T>::deserialize(deserializer)?;
        Ok(vec.into_iter().collect())
    }
}

pub struct XorLinkedListIter<T> {
    xor_linked_list: XorLinkedList<T>,
}
impl<T> Iterator for XorLinkedListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.xor_linked_list.pop_front()
    }
}

pub struct RefXorLinkedListIter<'a, T> {
    xor_linked_list_lifetime: PhantomData<&'a XorLinkedList<T>>,
    current_ptr: *mut XorNode<T>,
    prev_ptr: *mut XorNode<T>,
}
impl<'a, T> Iterator for RefXorLinkedListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_ptr.is_null() {
            return None;
        }
        unsafe {
            let payload_ref = &(*self.current_ptr).payload;
            let new_ptr = xor_ptrs((*self.current_ptr).xor_ptr, self.prev_ptr);
            self.prev_ptr = self.current_ptr;
            self.current_ptr = new_ptr;

            Some(payload_ref)
        }
    }
}

pub struct MutRefXorLinkedListIter<'a, T> {
    xor_linked_list_lifetime: PhantomData<&'a mut XorLinkedList<T>>,
    current_ptr: *mut XorNode<T>,
    prev_ptr: *mut XorNode<T>,
}
impl<'a, T> Iterator for MutRefXorLinkedListIter<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_ptr.is_null() {
            return None;
        }
        unsafe {
            let payload_ref = &mut (*self.current_ptr).payload;
            let new_ptr = xor_ptrs((*self.current_ptr).xor_ptr, self.prev_ptr);
            self.prev_ptr = self.current_ptr;
            self.current_ptr = new_ptr;

            Some(payload_ref)
        }
    }
}

pub struct ReverseXorLinkedListIter<T> {
    xor_linked_list: XorLinkedList<T>,
}
impl<T> Iterator for ReverseXorLinkedListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.xor_linked_list.pop_back()
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    #[test]
    fn test_push_and_iterate() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(2);
        list.push_front(1);
        list.push_back(3);

        let mut items: Vec<i32> = vec![];
        for i in list {
            items.push(i);
        }

        assert_eq!(1, items[0]);
        assert_eq!(2, items[1]);
        assert_eq!(3, items[2]);
    }

    #[test]
    fn test_iterate_ref() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        let mut items: Vec<i32> = vec![];
        for i in &list {
            items.push(*i);
        }

        assert_eq!(1, items[0]);
        assert_eq!(2, items[1]);
        assert_eq!(3, items[2]);
    }

    #[test]
    fn test_iterate_mut_ref() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        for i in &mut list {
            *i += 100;
        }

        let mut items: Vec<i32> = vec![];
        for i in &list {
            items.push(*i);
        }

        assert_eq!(101, items[0]);
        assert_eq!(102, items[1]);
        assert_eq!(103, items[2]);
    }

    #[test]
    fn test_get() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        assert_eq!(1, *list.get(0).unwrap());
        assert_eq!(2, *list.get(1).unwrap());
        assert_eq!(3, *list.get(2).unwrap());
        assert!(list.get(100).is_none());
    }

    #[test]
    fn test_get_mut() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        *list.get_mut(0).unwrap() += 100;
        *list.get_mut(1).unwrap() += 100;
        *list.get_mut(2).unwrap() += 100;

        assert_eq!(101, *list.get(0).unwrap());
        assert_eq!(102, *list.get(1).unwrap());
        assert_eq!(103, *list.get(2).unwrap());
        assert!(list.get_mut(100).is_none());
    }

    #[test]
    fn test_index() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        assert_eq!(1, list[0]);
        assert_eq!(2, list[1]);
        assert_eq!(3, list[2]);
    }

    #[test]
    fn test_index_mut() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        list[0] += 100;
        list[1] += 100;
        list[2] += 100;

        assert_eq!(101, list[0]);
        assert_eq!(102, list[1]);
        assert_eq!(103, list[2]);
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        list[100];
    }

    #[test]
    #[should_panic]
    fn test_mut_index_out_of_bounds() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        list[100] += 100;
    }

    #[test]
    fn test_into_reverse_iter() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(2);
        list.push_front(1);
        list.push_back(3);

        let mut items: Vec<i32> = vec![];
        for i in list.into_reverse_iter() {
            items.push(i);
        }

        assert_eq!(3, items[0]);
        assert_eq!(2, items[1]);
        assert_eq!(1, items[2]);
    }

    #[test]
    fn test_peek_front() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        assert!(list.peek_front().is_none());
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        assert_eq!(1, *list.peek_front().unwrap());
    }

    #[test]
    fn test_peek_back() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        assert!(list.peek_back().is_none());
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        assert_eq!(3, *list.peek_back().unwrap());
    }

    #[test]
    fn test_peek_front_mut() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        assert!(list.peek_front_mut().is_none());
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);
        *list.peek_front_mut().unwrap() += 100;

        assert_eq!(101, *list.peek_front().unwrap());
    }

    #[test]
    fn test_peek_back_mut() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        assert!(list.peek_back_mut().is_none());
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);
        *list.peek_back_mut().unwrap() += 100;

        assert_eq!(103, *list.peek_back().unwrap());
    }

    #[test]
    fn test_clone() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        let cloned_list = list.clone();
        assert_eq!(3, cloned_list.len());

        for (i, j) in list.into_iter().zip(cloned_list.into_iter()) {
            assert_eq!(i, j);
        }
    }

    #[test]
    fn test_reverse_iter() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        let mut iter = list.reverse_iter();
        assert_eq!(3, *iter.next().unwrap());
        assert_eq!(2, *iter.next().unwrap());
        assert_eq!(1, *iter.next().unwrap());
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_reverse_iter_mut() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        let mut iter = list.reverse_iter_mut();
        *iter.next().unwrap() += 300;
        *iter.next().unwrap() += 200;
        *iter.next().unwrap() += 100;
        assert!(iter.next().is_none());
        drop(iter);

        assert_eq!(101, list[0]);
        assert_eq!(202, list[1]);
        assert_eq!(303, list[2]);
    }

    #[test]
    fn test_extend() {
        let mut list1: XorLinkedList<i32> = XorLinkedList::new();
        list1.push_front(3);
        list1.push_front(2);
        list1.push_front(1);

        let mut list2: XorLinkedList<i32> = XorLinkedList::new();
        list2.push_front(5);
        list2.push_front(4);

        list1.extend(list2);

        assert_eq!(5, list1.len());
        assert_eq!(1, list1[0]);
        assert_eq!(2, list1[1]);
        assert_eq!(3, list1[2]);
        assert_eq!(4, list1[3]);
        assert_eq!(5, list1[4]);
    }

    #[test]
    fn test_drop() {
        const EXPECTED_DROP_COUNT: i32 = 5;
        let drop_counter = Rc::new(RefCell::new(0));
        struct DropImpl {
            drop_counter: Rc<RefCell<i32>>,
        }
        impl Drop for DropImpl {
            fn drop(&mut self) {
                *self.drop_counter.borrow_mut() += 1;
            }
        }

        let mut list = XorLinkedList::new();
        for _ in 0..EXPECTED_DROP_COUNT {
            list.push_back(DropImpl {
                drop_counter: drop_counter.clone(),
            });
        }
        drop(list);

        assert_eq!(EXPECTED_DROP_COUNT, *drop_counter.borrow());
    }

    #[test]
    fn test_reverse() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(3);
        list.push_front(2);
        list.push_front(1);

        list.reverse();

        assert_eq!(3, list[0]);
        assert_eq!(2, list[1]);
        assert_eq!(1, list[2]);
    }

    #[test]
    fn test_insert_at() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.insert_at(0, 4);
        list.insert_at(0, 1);
        list.insert_at(1, 2);
        list.insert_at(2, 3);
        list.insert_at(4, 7);
        list.insert_at(4, 6);
        list.insert_at(4, 5);

        assert_eq!(7, list.len());
        assert_eq!(1, list[0]);
        assert_eq!(2, list[1]);
        assert_eq!(3, list[2]);
        assert_eq!(4, list[3]);
        assert_eq!(5, list[4]);
        assert_eq!(6, list[5]);
        assert_eq!(7, list[6]);
    }

    #[test]
    #[should_panic]
    fn test_insert_panic() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.insert_at(0, 1);
        list.insert_at(3, 2);
    }

    #[test]
    fn test_insert_at_many_near_back() {
        let mut list = XorLinkedList::new();

        for i in 0..10 {
            list.push_back(i);
        }

        for i in 0..20 {
            let idx = list.len() - 1;
            list.insert_at(idx, 100 + i);
        }

        let collected: Vec<_> = list.into_iter().collect();

        assert_eq!(collected.first(), Some(&0));
        assert_eq!(collected.last(), Some(&9));
        assert_eq!(collected.len(), 30);

        let mut sorted = collected.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 30);
    }

    #[test]
    fn test_insert_at_forward_backward_consistency() {
        let mut list = XorLinkedList::new();

        for i in 0..15 {
            list.push_back(i);
        }

        let insert_positions = [7, 3, 10, 5, 12, 1, 8];
        for (i, &pos) in insert_positions.iter().enumerate() {
            list.insert_at(pos, 1000 + i);
        }

        let forward: Vec<_> = (&list).into_iter().cloned().collect();

        let backward: Vec<_> = list.reverse_iter().cloned().collect();

        let mut forward_reversed = forward.clone();
        forward_reversed.reverse();

        assert_eq!(forward_reversed, backward);
    }

    #[test]
    fn test_insert_at_stress_random_positions() {
        let mut list = XorLinkedList::new();

        for i in 0..50 {
            let pos = if list.is_empty() {
                0
            } else {
                (i * 7) % list.len()
            };
            list.insert_at(pos, i);
        }

        let forward: Vec<_> = (&list).into_iter().cloned().collect();
        let backward: Vec<_> = list.reverse_iter().cloned().collect();

        let mut reversed_forward = forward.clone();
        reversed_forward.reverse();

        assert_eq!(reversed_forward, backward);
    }

    #[test]
    fn test_insert_at_backward_off_by_one_detection() {
        let mut list = XorLinkedList::new();

        for i in 0..8 {
            list.push_back(i);
        }

        list.insert_at(1, 100);
        list.insert_at(3, 200);
        list.insert_at(5, 300);

        let idx = list.len() / 2 + 1;
        list.insert_at(idx, 999);

        assert_eq!(list[idx], 999);

        assert_eq!(list[idx - 1], 300);
        assert_eq!(list[idx + 1], 3);
    }

    #[test]
    fn test_remove_at_ends() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        assert!(list.remove_at(0).is_none());
        assert!(list.remove_at(1).is_none());
        assert_eq!(0, list.len());

        list.push_back(1);
        list.push_back(2);
        assert_eq!(1, list.remove_at(0).unwrap());
        assert_eq!(1, list.len());

        list.push_back(3);
        assert_eq!(3, list.remove_at(1).unwrap());
        assert_eq!(1, list.len());
        assert_eq!(2, list[0]);
    }

    #[test]
    fn test_remove_at_middle() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        for i in 0..10 {
            list.push_back(i);
        }

        assert_eq!(8, list.remove_at(8).unwrap());
        assert_eq!(9, list.len());

        assert_eq!(1, list.remove_at(1).unwrap());
        assert_eq!(8, list.len());

        assert_eq!(0, list[0]);
        assert_eq!(2, list[1]);
        assert_eq!(3, list[2]);
        assert_eq!(4, list[3]);
        assert_eq!(5, list[4]);
        assert_eq!(6, list[5]);
        assert_eq!(7, list[6]);
        assert_eq!(9, list[7]);
    }

    #[test]
    fn test_clear() {
        let mut list = XorLinkedList::new();
        list.clear();
        assert_eq!(0, list.size);

        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        list.clear();
        assert_eq!(0, list.size);
        assert!(list.peek_back().is_none());
    }

    #[test]
    fn test_equals() {
        let mut list1 = XorLinkedList::new();
        list1.push_back(1);
        list1.push_back(2);
        let mut list2 = list1.clone();
        assert_eq!(list1, list2);

        list2.push_back(3);
        assert_ne!(list1, list2);

        list1.push_back(3);
        assert_eq!(list1, list2);

        list1[1] = 5;
        assert_ne!(list1, list2);
    }

    #[test]
    fn test_from_iterator() {
        let list = XorLinkedList::from_iter([1, 2, 3].into_iter());

        assert_eq!(3, list.len());
        assert_eq!(1, list[0]);
        assert_eq!(2, list[1]);
        assert_eq!(3, list[2]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        let mut list = XorLinkedList::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let serialized = serde_json::to_string(&list).unwrap();

        let deserialized: XorLinkedList<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(list, deserialized);
    }
}
