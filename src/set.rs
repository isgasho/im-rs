use std::sync::Arc;
use std::iter::{IntoIterator, FromIterator};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter, Error};
use std::collections::{HashSet, BTreeSet};
use std::hash::Hash;
use map::{self, Map};

#[macro_export]
macro_rules! set {
    () => { $crate::set::Set::empty() };

    ( $($x:expr),* ) => {{
        let mut l = $crate::set::Set::empty();
        $(
            l = l.insert($x);
        )*
            l
    }};
}

pub struct Set<A>(Map<A, ()>);

impl<A> Set<A> {
    pub fn empty() -> Self {
        Set(Map::empty())
    }

    pub fn singleton(a: A) -> Self {
        Set(Map::singleton(a, ()))
    }

    pub fn iter(&self) -> Iter<A> {
        Iter { it: self.0.iter() }
    }

    pub fn size(&self) -> usize {
        self.0.size()
    }

    pub fn lookup_min(&self) -> Option<Arc<A>> {
        self.0.lookup_min().map(|(a, _)| a)
    }

    pub fn lookup_max(&self) -> Option<Arc<A>> {
        self.0.lookup_max().map(|(a, _)| a)
    }

    pub fn valid(&self) -> bool {
        self.0.valid()
    }
}

impl<A: Clone> Set<A> {
    pub fn clone_iter(&self) -> Cloned<A> {
        Cloned { it: self.iter() }
    }
}

impl<A: Ord> Set<A> {
    pub fn insert(&self, a: A) -> Self {
        Set(self.0.insert(a, ()))
    }

    pub fn insert_ref(&self, a: Arc<A>) -> Self {
        Set(self.0.insert_ref(a, Arc::new(())))
    }

    pub fn contains(&self, a: &A) -> bool {
        self.0.contains_key(a)
    }

    pub fn delete(&self, a: &A) -> Self {
        Set(self.0.delete(a))
    }

    pub fn union(&self, other: &Self) -> Self {
        Set(self.0.union(&other.0))
    }

    pub fn unions<I>(i: I) -> Self where I: IntoIterator<Item = Self> {
        i.into_iter().fold(set![], |a, b| a.union(&b))
    }

    pub fn difference(&self, other: &Self) -> Self {
        Set(self.0.difference(&other.0))
    }

    pub fn intersection(&self, other: &Self) -> Self {
        Set(self.0.intersection(&other.0))
    }

    pub fn split(&self, split: &A) -> (Self, Self) {
        let (l, r) = self.0.split(split);
        (Set(l), Set(r))
    }

    pub fn split_member(&self, split: &A) -> (Self, bool, Self) {
        let (l, m, r) = self.0.split_lookup(split);
        (Set(l), m.is_some(), Set(r))
    }

    pub fn is_subset(&self, other: &Self) -> bool {
        self.0.is_submap(&other.0)
    }

    pub fn is_proper_subset(&self, other: &Self) -> bool {
        self.0.is_proper_submap(&other.0)
    }

    pub fn take(&self, n: usize) -> Self {
        Set(self.0.take(n))
    }

    pub fn drop(&self, n: usize) -> Self {
        Set(self.0.drop(n))
    }

    pub fn pop_min(&self) -> (Option<Arc<A>>, Self) {
        let (pair, set) = self.0.pop_min_with_key();
        (pair.map(|(a, _)| a), Set(set))
    }

    pub fn pop_max(&self) -> (Option<Arc<A>>, Self) {
        let (pair, set) = self.0.pop_max_with_key();
        (pair.map(|(a, _)| a), Set(set))
    }

    pub fn delete_min(&self) -> Self {
        self.pop_min().1
    }

    pub fn delete_max(&self) -> Self {
        self.pop_max().1
    }
}

// Core traits

impl<A> Clone for Set<A> {
    fn clone(&self) -> Self {
        Set(self.0.clone())
    }
}

impl<A: PartialEq> PartialEq for Set<A> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A: Eq> Eq for Set<A> {}

impl<A: PartialOrd> PartialOrd for Set<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<A: Ord> Ord for Set<A> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<A> Default for Set<A> {
    fn default() -> Self {
        set![]
    }
}

impl<A: Debug> Debug for Set<A> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{{ ")?;
        let mut it = self.iter().peekable();
        loop {
            match it.next() {
                None => break,
                Some(a) => {
                    write!(f, "{:?}", a)?;
                    match it.peek() {
                        None => write!(f, " }}")?,
                        Some(_) => write!(f, ", ")?,
                    }
                }
            }
        }
        Ok(())
    }
}

// Iterators

pub struct Iter<A> {
    it: map::Iter<A, ()>
}

impl<A> Iterator for Iter<A> {
    type Item = Arc<A>;

    fn next(&mut self) -> Option<Self::Item> {
        self.it.next().map(|(a, _)| a)
    }
}

pub struct Cloned<A> {
    it: Iter<A>,
}

impl<A: Clone> Iterator for Cloned<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        match self.it.next() {
            None => None,
            Some(v) => Some((*v).clone()),
        }
    }
}

impl<A: Ord> FromIterator<A> for Set<A> {
    fn from_iter<T>(i: T) -> Self
        where T: IntoIterator<Item = A>
    {
        i.into_iter().fold(set![], |s, a| s.insert(a))
    }
}

impl<A: Ord> FromIterator<Arc<A>> for Set<A> {
    fn from_iter<T>(i: T) -> Self
        where T: IntoIterator<Item = Arc<A>>
    {
        i.into_iter()
            .fold(set![], |s, a| s.insert_ref(a))
    }
}

impl<'a, A> IntoIterator for &'a Set<A> {
    type Item = Arc<A>;
    type IntoIter = Iter<A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<A> IntoIterator for Set<A> {
    type Item = Arc<A>;
    type IntoIter = Iter<A>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// Conversions

impl<'a, A: Ord + Clone> From<&'a [A]> for Set<A> {
    fn from(slice: &'a [A]) -> Self {
        slice.into_iter().cloned().collect()
    }
}

impl<'a, A: Ord> From<&'a [Arc<A>]> for Set<A> {
    fn from(slice: &'a [Arc<A>]) -> Self {
        slice.into_iter().cloned().collect()
    }
}

impl<A: Ord> From<Vec<A>> for Set<A> {
    fn from(vec: Vec<A>) -> Self {
        vec.into_iter().collect()
    }
}

impl<'a, A: Ord + Clone> From<&'a Vec<A>> for Set<A> {
    fn from(vec: &Vec<A>) -> Self {
        vec.into_iter().cloned().collect()
    }
}

impl<'a, A: Ord> From<&'a Vec<Arc<A>>> for Set<A> {
    fn from(vec: &Vec<Arc<A>>) -> Self {
        vec.into_iter().cloned().collect()
    }
}

impl<A: Eq + Hash + Ord> From<HashSet<A>> for Set<A> {
    fn from(hash_set: HashSet<A>) -> Self {
        hash_set.into_iter().collect()
    }
}

impl<'a, A: Eq + Hash + Ord + Clone> From<&'a HashSet<A>> for Set<A> {
    fn from(hash_set: &HashSet<A>) -> Self {
        hash_set.into_iter().cloned().collect()
    }
}

impl<'a, A: Eq + Hash + Ord> From<&'a HashSet<Arc<A>>> for Set<A> {
    fn from(hash_set: &HashSet<Arc<A>>) -> Self {
        hash_set.into_iter().cloned().collect()
    }
}

impl<A: Ord> From<BTreeSet<A>> for Set<A> {
    fn from(btree_set: BTreeSet<A>) -> Self {
        btree_set.into_iter().collect()
    }
}

impl<'a, A: Ord + Clone> From<&'a BTreeSet<A>> for Set<A> {
    fn from(btree_set: &BTreeSet<A>) -> Self {
        btree_set.into_iter().cloned().collect()
    }
}

impl<'a, A: Ord> From<&'a BTreeSet<Arc<A>>> for Set<A> {
    fn from(btree_set: &BTreeSet<Arc<A>>) -> Self {
        btree_set.into_iter().cloned().collect()
    }
}

// QuickCheck

#[cfg(any(test, feature = "quickcheck"))]
use quickcheck::{Arbitrary, Gen};

#[cfg(any(test, feature = "quickcheck"))]
impl<A: Ord + Arbitrary + Sync> Arbitrary for Set<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Set::from_iter(Vec::<A>::arbitrary(g))
    }
}
