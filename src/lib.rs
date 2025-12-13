use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
    ptr::null_mut,
};

const INDEX_BOUNDS_ERROR: &str = "Index is out of bounds";

#[inline(always)]
fn xor_next_ptr<T>(
    combined_xor_ptr: *mut XorNode<T>,
    other_ptr: *mut XorNode<T>,
) -> *mut XorNode<T> {
    let combined_ptr_value = combined_xor_ptr as usize;
    let other_ptr_value = other_ptr as usize;
    let new_ptr_value = combined_ptr_value ^ other_ptr_value;
    new_ptr_value as _
}

struct XorNode<T> {
    payload: T,
    xor_ptr: *mut XorNode<T>,
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
                new_ptr = xor_next_ptr((*current_ptr).xor_ptr, prev_ptr);
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

    unsafe fn push_end(end_ptr1: &mut *mut XorNode<T>, end_ptr2: &mut *mut XorNode<T>, value: T) {
        let new_node = Box::leak(Box::new(XorNode {
            payload: value,
            xor_ptr: null_mut(),
        }));

        if end_ptr2.is_null() {
            debug_assert!(end_ptr1.is_null());
            *end_ptr1 = new_node;
            *end_ptr2 = new_node;
        } else {
            unsafe {
                (**end_ptr2).xor_ptr = xor_next_ptr((**end_ptr2).xor_ptr, new_node);
            }
            new_node.xor_ptr = *end_ptr2;
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

    unsafe fn pop_end(end_ptr1: &mut *mut XorNode<T>, end_ptr2: &mut *mut XorNode<T>) -> Option<T> {
        if end_ptr1.is_null() {
            debug_assert!(end_ptr2.is_null());
            return None;
        }
        let old_ptr = *end_ptr1;
        unsafe {
            if end_ptr1 == end_ptr2 {
                *end_ptr1 = null_mut();
                *end_ptr2 = null_mut();
            } else {
                *end_ptr1 = (**end_ptr1).xor_ptr;
                (**end_ptr1).xor_ptr = xor_next_ptr((**end_ptr1).xor_ptr, old_ptr);
            }
            Some(Box::from_raw(old_ptr).payload)
        }
    }

    /// removes and returns the element from the start of the list
    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.size = self.size.saturating_sub(1);
            Self::pop_end(&mut self.start, &mut self.end)
        }
    }

    /// removes and returns the element from the end of the list
    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.size = self.size.saturating_sub(1);
            Self::pop_end(&mut self.end, &mut self.start)
        }
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
            _xor_linked_list: self,
            current_ptr: self.end,
            prev_ptr: null_mut(),
        }
    }

    /// returns an iterator of mutable element references from the end to the start of the list
    pub fn reverse_iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        let current_ptr = self.end;
        MutRefXorLinkedListIter {
            _xor_linked_list: self,
            current_ptr,
            prev_ptr: null_mut(),
        }
    }

    /// reverses the order of the list
    pub fn reverse(&mut self) {
        let new_start = self.end;
        self.end = self.start;
        self.start = new_start;
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
        loop {
            if self.pop_front().is_none() {
                return;
            }
        }
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
            _xor_linked_list: self,
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
            _xor_linked_list: self,
            current_ptr,
            prev_ptr,
        }
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
    _xor_linked_list: &'a XorLinkedList<T>,
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
            let new_ptr = xor_next_ptr((*self.current_ptr).xor_ptr, self.prev_ptr);
            self.prev_ptr = self.current_ptr;
            self.current_ptr = new_ptr;

            Some(payload_ref)
        }
    }
}

pub struct MutRefXorLinkedListIter<'a, T> {
    _xor_linked_list: &'a mut XorLinkedList<T>,
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
            let new_ptr = xor_next_ptr((*self.current_ptr).xor_ptr, self.prev_ptr);
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
}
