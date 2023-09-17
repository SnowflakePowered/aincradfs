use std::borrow::Borrow;
use crate::old_path::{OwnedProjectedPath, ProjectedPath};

pub struct PathTrie<V>(qp_trie::Trie<OwnedProjectedPath, V>);


impl<V> PathTrie<V> {
    pub fn insert(&mut self, path: OwnedProjectedPath, value: V) {
    }
}