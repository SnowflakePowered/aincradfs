use std::borrow::Borrow;

use bstr::{BStr, ByteSlice};
use bytemuck::Pod;
use qp_trie::Break;
use widestring::U16Str;

mod components;
mod u16path;
mod u8path;

pub use u16path::{U16Path, U16PathBuf};
pub use u8path::{U8Path, U8PathBuf};
pub use components::{Components, Component};

pub trait PathBuf {
    fn new() -> Self;
    fn push<A>(&mut self, entry: A)
    where
        A: Borrow<A>;

    fn pop(&mut self);
}

pub trait PathStr: 'static + PartialEq {
    type ComponentType: Copy + PartialEq + Pod;

    fn as_slice(&self) -> &[Self::ComponentType];
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn from_slice(slice: &[Self::ComponentType]) -> &Self;
}

impl PathStr for U16Str {
    type ComponentType = u16;

    fn as_slice(&self) -> &[Self::ComponentType] {
        U16Str::as_slice(self)
    }

    fn len(&self) -> usize {
        U16Str::len(self)
    }

    fn is_empty(&self) -> bool {
        U16Str::is_empty(self)
    }

    fn from_slice(slice: &[Self::ComponentType]) -> &Self {
        U16Str::from_slice(slice)
    }
}

impl PathStr for BStr {
    type ComponentType = u8;

    fn as_slice(&self) -> &[Self::ComponentType] {
        self.as_bytes()
    }

    fn len(&self) -> usize {
        self.as_bytes().len()
    }

    fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    fn from_slice(slice: &[Self::ComponentType]) -> &Self {
        BStr::new(slice)
    }
}

pub trait PathOwned: Break + Clone + Borrow<[u8]> {
    type Borrowed: Path + ?Sized;

    fn new() -> Self;
    fn push(&mut self, component: &<Self::Borrowed as Path>::Str);
    fn pop(&mut self);
}

pub trait Path: PartialEq + Eq {
    type Str: PathStr + ?Sized;

    const CURRENT_DIR: &'static Self::Str;
    const PARENT_DIR: &'static Self::Str;
    const SEPARATOR: &'static Self::Str;

    fn is_separator(t: <Self::Str as PathStr>::ComponentType) -> bool;

    fn root() -> &'static Self;
    fn empty() -> &'static Self;

    fn has_root(&self) -> bool;
    fn components(&self) -> Components<Self>;

    fn from_str(str: &Self::Str) -> &Self;
}
