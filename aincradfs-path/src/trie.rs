use std::borrow::Borrow;
use crate::old_path::{OwnedProjectedPath, ProjectedPath};
use crate::path::{Path, PathOwned, PathStr};

pub struct PathTrie<K: PathOwned, V>(qp_trie::Trie<K, V>);


impl<K: PathOwned, V> PathTrie<K, V> {
    pub fn insert(&mut self, path: OwnedProjectedPath, value: V) {
    }
}