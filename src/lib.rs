use std::collections::VecDeque;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Index {
    slot: u32,
    generation: u32,
}

impl Index {
    pub fn into_raw(&self) -> u64 {
        ((self.slot as u64) << 32) | (self.generation as u64)
    }
}

impl std::fmt::Debug for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Index({}:{})", self.slot, self.generation))?;
        Ok(())
    }
}

pub struct GenArray<T> {
    slots: Vec<(u32, Option<T>)>,
    empty: VecDeque<u32>,
}

impl<T> GenArray<T> {
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            empty: VecDeque::new(),
        }
    }

    pub fn get(&self, index: Index) -> Option<&T> {
        let slot = index.slot as usize;

        if let Some((gen, Some(value))) = self.slots.get(slot) {
            if index.generation == *gen {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T> {
        let slot = index.slot as usize;

        if let Some((gen, Some(value))) = self.slots.get_mut(slot) {
            if index.generation == *gen {
                Some(value)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn push(&mut self, value: T) -> Index {
        if let Some(index) = self.empty.pop_front() {
            self.slots[index as usize] = (self.slots[index as usize].0 + 1, Some(value));
            Index {
                slot: index,
                generation: self.slots[index as usize].0,
            }
        } else {
            self.slots.push((0, Some(value)));
            Index {
                slot: (self.slots.len() - 1).try_into().unwrap(),
                generation: 0,
            }
        }
    }

    pub fn remove(&mut self, index: Index) {
        if let Some((generation, value)) = self.slots.get_mut(index.slot as usize) {
            if *generation == index.generation && value.is_some() {
                *value = None;
                self.empty.push_back(index.slot);
            }
        }
    }

    pub fn iter(&self) -> GenArrayIterator<'_, T> {
        GenArrayIterator { arr: self, pos: 0 }
    }

    pub fn iter_mut(&mut self) -> GenArrayIteratorMut<'_, T> {
        GenArrayIteratorMut { arr: self, pos: 0 }
    }
}

impl<T> Default for GenArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct GenArrayIteratorMut<'a, T> {
    arr: &'a mut GenArray<T>,
    pos: usize,
}

impl<'a, T> Iterator for GenArrayIteratorMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.arr.slots.len() {
            None
        } else if let Some(next_occupied_slot) = self.arr.slots[self.pos..]
            .iter()
            .position(|s| s.1.is_some())
        {
            self.pos += next_occupied_slot;

            self.pos += 1;
            Some(unsafe { &mut *(self.arr.slots[self.pos - 1].1.as_mut().unwrap() as *mut T) })
        } else {
            None
        }
    }
}

pub struct GenArrayIterator<'a, T> {
    arr: &'a GenArray<T>,
    pos: usize,
}

impl<'a, T> Iterator for GenArrayIterator<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.arr.slots.len() {
            None
        } else if let Some(next_occupied_slot) = self.arr.slots[self.pos..]
            .iter()
            .position(|s| s.1.is_some())
        {
            self.pos += next_occupied_slot;

            self.pos += 1;
            Some(unsafe { &*(self.arr.slots[self.pos - 1].1.as_ref().unwrap() as *const T) })
        } else {
            None
        }
    }
}

impl<'a, T> IntoIterator for &'a GenArray<T> {
    type Item = &'a T;

    type IntoIter = GenArrayIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut GenArray<T> {
    type Item = &'a mut T;

    type IntoIter = GenArrayIteratorMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iter() {
        let mut arr = GenArray::new();

        let _ = arr.push("hello");
        let b = arr.push("world");
        let c = arr.push("lorem");
        let _ = arr.push("ipsum");
        arr.remove(b);
        arr.remove(c);

        assert_eq!(
            vec!["hello", "ipsum"],
            arr.iter().cloned().collect::<Vec<&'static str>>()
        );
    }

    #[test]
    fn iter_mut() {
        let mut arr = GenArray::new();

        let _ = arr.push(2);
        let b = arr.push(3);
        let c = arr.push(5);
        let _ = arr.push(8);
        arr.remove(b);
        arr.remove(c);

        for number in &mut arr {
            *number *= 2;
        }

        assert_eq!(vec![4, 16], arr.iter().cloned().collect::<Vec<u32>>());
    }

    #[test]
    fn works() {
        let mut arr = GenArray::new();

        let a = arr.push(String::from("hello"));
        let b = arr.push(String::from("world"));
        let c = arr.push(String::from("lorem"));
        let d = arr.push(String::from("ipsum"));
        arr.remove(b);
        arr.remove(c);
        let e = arr.push(String::from("dolor"));
        let f = arr.push(String::from("sit"));
        let g = arr.push(String::from("amet"));
        arr.remove(c); // trying to remove something for the second
                       // time should just do nothing and not for example
                       // panic

        *arr.get_mut(f).unwrap() = String::from("quanti");

        assert_eq!(arr.get(a), Some(&"hello".into()));
        assert_eq!(arr.get(b), None);
        assert_eq!(arr.get(c), None);
        assert_eq!(arr.get(d), Some(&"ipsum".into()));
        assert_eq!(arr.get(e), Some(&"dolor".into()));
        assert_eq!(arr.get(f), Some(&"quanti".into()));
        assert_eq!(arr.get(g), Some(&"amet".into()));

        assert_eq!(a.generation, 0);
        assert_eq!(b.generation, 0);
        assert_eq!(c.generation, 0);
        assert_eq!(d.generation, 0);
        assert_eq!(e.generation, 1);
        assert_eq!(f.generation, 1);
        assert_eq!(g.generation, 0);

        assert_eq!(a.slot, 0);
        assert_eq!(b.slot, 1);
        assert_eq!(c.slot, 2);
        assert_eq!(d.slot, 3);
        assert_eq!(e.slot, 1);
        assert_eq!(f.slot, 2);
        assert_eq!(g.slot, 4);
    }
}
