use std::borrow::Borrow;
use qp_trie::Break;
use crate::path::{Path, PathOwned, PathStr};

#[derive(Debug)]
pub struct PathTrie<K: PathOwned, V>(qp_trie::Trie<K, V>);


impl<K: PathOwned, V> PathTrie<K, V>
    where <K as Break>::Split: AsRef<[u8]>
{
    pub fn new() -> Self {
        Self(qp_trie::Trie::new())
    }
    pub fn insert(&mut self, path: K, value: V) -> Option<V> {
        self.0.insert(path, value)
    }

    pub fn longest_prefix(&self, path: K) -> &K::Borrowed {
        // SAFETY: all implementations of break are [u8]
        // TODO: break on component
        let pref = self.0.longest_common_prefix(&path).as_ref();
        let str = <K::Borrowed as Path>::Str::from_slice(bytemuck::cast_slice(pref));
        K::Borrowed::from_str(str)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;
    use widestring::U16Str;
    use crate::path::{U16Path, U16PathBuf};
    use crate::trie::PathTrie;

    #[test]
    pub fn insert() {
        let mut trie = PathTrie::new();
        trie.insert(U16PathBuf::from("/hello/world"), 1);
        trie.insert(U16PathBuf::from("/hello/world/spam"), 2);
        trie.insert(U16PathBuf::from("/hello/spam/eggs"), 1);

        let pref = trie.longest_prefix(U16PathBuf::from("/hello/world/nope"));
        println!("{:?}", trie);

        println!("{:?}", pref);
    }
}