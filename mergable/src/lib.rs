pub use merge_derive::MergeFrom;
extern crate alloc;
use alloc::collections::BTreeMap;

pub trait MergeFrom {
    fn merge_from(&mut self, other: &Self);
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

impl_primitive!(u8, i8, u16, i16, u32, i32, u64, i64, f32, f64, bool);

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

//vecs append
impl<T: Clone> MergeFrom for Vec<T> {
    fn merge_from(&mut self, other: &Self) {
        self.extend_from_slice(other);
    }
}
//boxes merge the internal 'T'
impl<T: Clone + MergeFrom> MergeFrom for Box<T> {
    fn merge_from(&mut self, other: &Self) {
        MergeFrom::merge_from(self.as_mut(), other.as_ref());
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
