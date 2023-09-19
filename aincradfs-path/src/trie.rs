use crate::path::PathOwned;
use qp_trie::Break;
use std::borrow::Borrow;

#[derive(Debug)]
pub struct PathTrie<K: PathOwned, V>(qp_trie::Trie<K, V>);

impl<K: PathOwned, V> PathTrie<K, V>
where
    <K as Break>::Split: Borrow<K::Borrowed>,
{
    pub fn new() -> Self {
        Self(qp_trie::Trie::new())
    }
    pub fn insert(&mut self, path: K, value: V) -> Option<V> {
        self.0.insert(path, value)
    }

    pub fn longest_prefix(&self, path: K) -> &K::Borrowed {
        let pref = self.0.longest_common_prefix(&path).borrow();
        pref
    }
}

#[cfg(test)]
mod test {
    use crate::path::{Path, U16Path, U16PathBuf, U8Path, U8PathBuf};
    use crate::trie::PathTrie;
    use bstr::BStr;
    use widestring::u16str;

    #[test]
    pub fn insert_u16() {
        let mut trie = PathTrie::new();
        trie.insert(U16PathBuf::from("/hello/world"), 1);
        trie.insert(U16PathBuf::from("/hello/world/spam"), 2);
        trie.insert(U16PathBuf::from("/hello/spam/eggs"), 1);

        let pref = trie.longest_prefix(U16PathBuf::from("/hello/world/spad"));
        assert_eq!(U16Path::from_str(u16str!("/hello/world")), pref)
    }

    #[test]
    pub fn insert_u8() {
        let mut trie = PathTrie::new();
        trie.insert(U8PathBuf::from("/hello/world"), 1);
        trie.insert(U8PathBuf::from("/hello/world/spam"), 2);
        trie.insert(U8PathBuf::from("/hello/spam/eggs"), 1);

        let pref = trie.longest_prefix(U8PathBuf::from("/hello/world/spad"));
        assert_eq!(U8Path::from_str(BStr::new(b"/hello/world")), pref)
    }
}
