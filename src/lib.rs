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

pub struct XorLinkedList<T> {
    size: usize,
    start: *mut XorNode<T>,
    end: *mut XorNode<T>,
}
impl<T> XorLinkedList<T> {
    pub fn new() -> Self {
        Self {
            size: 0,
            start: null_mut(),
            end: null_mut(),
        }
    }

    pub fn peek_front(&self) -> Option<&T> {
        if self.size == 0 {
            None
        } else {
            unsafe { Some(&(*self.start).payload) }
        }
    }

    pub fn peek_front_mut(&self) -> Option<&mut T> {
        if self.size == 0 {
            None
        } else {
            unsafe { Some(&mut (*self.start).payload) }
        }
    }

    pub fn peek_back(&self) -> Option<&T> {
        if self.size == 0 {
            None
        } else {
            unsafe { Some(&(*self.end).payload) }
        }
    }

    pub fn peek_back_mut(&self) -> Option<&mut T> {
        if self.size == 0 {
            None
        } else {
            unsafe { Some(&mut (*self.end).payload) }
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.size {
            return None;
        }

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

        Some(unsafe { &(*current_ptr).payload })
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.size {
            return None;
        }

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

        Some(unsafe { &mut (*current_ptr).payload })
    }

    pub fn len(&self) -> usize {
        self.size
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

    pub fn push_back(&mut self, value: T) {
        self.size += 1;
        unsafe {
            Self::push_end(&mut self.start, &mut self.end, value);
        }
    }

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
        let old_start = *end_ptr1;
        unsafe {
            if end_ptr1 == end_ptr2 {
                *end_ptr1 = null_mut();
                *end_ptr2 = null_mut();
            } else {
                *end_ptr1 = (**end_ptr1).xor_ptr;
                (**end_ptr1).xor_ptr = xor_next_ptr((**end_ptr1).xor_ptr, old_start);
            }
            Some(Box::from_raw(old_start).payload)
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.size = self.size.saturating_sub(1);
            Self::pop_end(&mut self.start, &mut self.end)
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.size = self.size.saturating_sub(1);
            Self::pop_end(&mut self.end, &mut self.start)
        }
    }

    pub fn into_iter_reverse(self) -> impl Iterator<Item = T> {
        ReverseXorLinkedListIter {
            xor_linked_list: self,
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
        f.debug_list().entries(self.into_iter()).finish()
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
    use super::*;

    #[test]
    fn test_add_iterate() {
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
    fn test_add_reverse_iterate() {
        let mut list: XorLinkedList<i32> = XorLinkedList::new();
        list.push_front(2);
        list.push_front(1);
        list.push_back(3);

        let mut items: Vec<i32> = vec![];
        for i in list.into_iter_reverse() {
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
}
