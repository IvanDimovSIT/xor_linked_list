use std::ptr::null_mut;

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
            self.size = self.size.wrapping_sub(1);
            Self::pop_end(&mut self.start, &mut self.end)
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.size = self.size.wrapping_sub(1);
            Self::pop_end(&mut self.end, &mut self.start)
        }
    }

    pub fn into_iter_reverse(self) -> impl Iterator<Item = T> {
        ReverseXorLinkedListIter {
            xor_linked_list: self,
        }
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

pub struct XorLinkedListIter<T> {
    xor_linked_list: XorLinkedList<T>,
}
impl<T> Iterator for XorLinkedListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.xor_linked_list.pop_front()
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
    fn test_memory() {
        for _ in 0..10000 {
            let mut list: XorLinkedList<u128> = XorLinkedList::new();
            for i in 0..10000u128 {
                list.push_front(i);
            }
        }
    }

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
}
