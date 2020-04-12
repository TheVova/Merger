#![no_std]

#[cfg(any(feature = "std", test))]
#[macro_use]
extern crate std;

#[cfg(any(feature = "std", test))]
use std::{
    boxed::Box,
    collections::BTreeMap,
    hash::Hash,
    string::String,
    vec::{self, Vec},
};

#[cfg(not(any(feature = "std", test)))]
#[macro_use]
extern crate alloc;

#[cfg(not(any(feature = "std", test)))]
use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::String,
    vec::{self, Vec},
};

pub use merge_derive::MergeFrom;

pub trait MergeFrom<Rhs = Self>
where
    Rhs: ?Sized,
{
    fn merge_from(&mut self, other: &Rhs);
    fn merge_with(&mut self, other: &Rhs, mut f: impl FnMut(&mut Self, &Rhs)) {
        f(self, other);
    }
}

// all primitive scalar types just get replaced with the latter value
macro_rules! impl_primitive {
    ($($s:ty),*) => {
        $(impl MergeFrom for $s {
            fn merge_from(&mut self, other: &Self) {
                *self = *other;
            }
        })*
    };
}

impl_primitive!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, bool, isize, usize);

// recurse into the 'T' in option and merge that, if available.
// is self is none clone the other.
impl<T: MergeFrom + Clone> MergeFrom for Option<T> {
    fn merge_from(&mut self, other: &Self) {
        if other.is_some() {
            let o = other.clone().unwrap();
            match self {
                None => {
                    self.replace(o);
                }
                Some(r) => MergeFrom::merge_from(r, &o),
            }
        }
    }
}

// recurse into the 'T' in option and merge that, if available.
// is self is none clone the other.
impl<T: MergeFrom + Clone> MergeFrom<T> for Option<T> {
    fn merge_from(&mut self, other: &T) {
        match self {
            Some(r) => MergeFrom::merge_from(r, &other),
            None => {
                self.replace(other.clone());
            }
        }
    }
}

//vecs append
impl<T, S> MergeFrom<S> for Vec<T>
where
    S: AsRef<[T]>,
    T: Clone,
{
    fn merge_from(&mut self, other: &S) {
        self.extend_from_slice(other.as_ref());
    }
}

//strings also append
impl<S: AsRef<str>> MergeFrom<S> for String {
    fn merge_from(&mut self, other: &S) {
        self.push_str(other.as_ref())
    }
}

//boxes merge the internal 'T'
impl<T: MergeFrom> MergeFrom for Box<T> {
    fn merge_from(&mut self, other: &Self) {
        MergeFrom::merge_from(self.as_mut(), other.as_ref());
    }
}

//boxes merge the internal 'T'
impl<T: MergeFrom> MergeFrom<T> for Box<T> {
    fn merge_from(&mut self, other: &T) {
        MergeFrom::merge_from(self.as_mut(), other);
    }
}

// hashmaps add keys and values from 'other'. if key exists in both, runs merge on the value.
impl<K: Eq + Ord + Clone, V: MergeFrom + Clone> MergeFrom<(K,V)> for BTreeMap<K, V> {
    fn merge_from(&mut self, (k,v): &(K,V)) {
            self.entry(k.clone())
                .and_modify(|f| MergeFrom::merge_from(f, v))
                .or_insert(v.clone());
    }
}

// hashmaps add keys and values from 'other'. if key exists in both, runs merge on the value.
impl<K: Eq + Ord + Clone, V: MergeFrom + Clone> MergeFrom for BTreeMap<K, V> {
    fn merge_from(&mut self, other: &Self) {
        for (k, v) in other.iter() {
            self.entry(k.clone())
                .and_modify(|f| MergeFrom::merge_from(f, v))
                .or_insert(v.clone());
        }
    }
}

#[cfg(feature = "std")]
impl<K: Eq + Hash + Clone, V: MergeFrom + Clone> MergeFrom for ::std::collections::HashMap<K, V> {
    fn merge_from(&mut self, other: &Self) {
        for (k, v) in other.iter() {
            self.entry(k.clone())
                .and_modify(|f| MergeFrom::merge_from(f, v))
                .or_insert(v.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(MergeFrom, Debug, PartialEq)]
    struct MergeMe {
        a: i32,
        b: Option<u8>,
        c: Vec<f64>,
        d: Box<Option<usize>>,
        e: Option<BTreeMap<usize, String>>,
    }

    #[derive(MergeFrom, Debug, PartialEq)]
    struct Scalars {
        a: i32,
        b: Option<f64>,
        c: Box<usize>,
    }

    #[derive(MergeFrom, Debug, PartialEq)]
    struct VecStr {
        a: Vec<i64>,
        b: String,
    }

    #[derive(MergeFrom, Debug, PartialEq)]

    struct Maps {
        a: BTreeMap<String, f64>,
        #[cfg(feature = "std")]
        b: std::collections::HashMap<String, f64>,
    }

    #[test]
    fn test_merge() {
        let mut base = MergeMe {
            a: 42,
            b: None,
            c: vec![1.0, 2.0, 3.0],
            d: Box::new(Some(42)),
            e: None,
        };
        let new = MergeMe {
            a: 0,
            b: Some(8),
            c: vec![3.0, 2.0, 1.0],
            d: Box::new(None),
            e: Some(BTreeMap::new()),
        };
        base.merge_from(&new);
        println!("{:?}", &base)
    }

    #[test]
    fn test_scalars() {
        let mut base = Scalars {
            a: 1,
            b: Some(2.0),
            c: Box::new(3),
        };
        let new = Scalars {
            a: 3,
            b: None,
            c: Box::new(1),
        };

        let expected = Scalars {
            a: 3,
            b: Some(2.0),
            c: Box::new(1),
        };

        base.merge_from(&new);
        assert_eq!(base, expected)
    }

    #[test]
    fn test_vec_str() {
        let mut v1 = vec![1, 2, 3];
        let v2 = [3, 2, 1];
        v1.merge_from(&v2);
        println!("{:?}", &v1);
        let mut base = VecStr {
            a: vec![1],
            b: String::from("Hello, "),
        };
        let new = VecStr {
            a: vec![2, 3],
            b: String::from("World"),
        };
        let expected = VecStr {
            a: vec![1, 2, 3],
            b: String::from("Hello, World"),
        };

        base.merge_from(&new);
        assert_eq!(base, expected)
    }
}
