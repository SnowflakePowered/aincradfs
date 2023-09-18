use std::borrow::Borrow;
use std::mem::ManuallyDrop;
use std::ops::{Deref, Index};
use std::slice::SliceIndex;
use bstr::{BStr, BString, ByteSlice, ByteVec};
use bytemuck::Pod;
use qp_trie::Break;
use widestring::{U16Str, u16str, U16String};
use crate::path::components::{Component, Components};

mod u8path;
mod u16path;
mod components;

pub use u8path::{U8PathBuf, U8Path};
pub use u16path::{U16PathBuf, U16Path};

pub trait PathBuf {
    fn new() -> Self;
    fn push<A>(&mut self, entry: A)
        where A: Borrow<A>;

    fn pop(&mut self);
}

pub(crate) trait PathStr: 'static + PartialEq
{
    type ComponentType: Copy + PartialEq + Pod;

    fn is_separator(t: Self::ComponentType) -> bool;
    fn as_slice(&self) -> &[Self::ComponentType];
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn from_slice(slice: &[Self::ComponentType]) -> &Self;
}

impl PathStr for U16Str {
    type ComponentType = u16;
    fn is_separator(t: Self::ComponentType) -> bool {
        [b'/' as u16, b'\\' as u16].contains(&t)
    }

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
    fn is_separator(t: Self::ComponentType) -> bool {
        [b'/', b'\\'].contains(&t)
    }

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

pub trait Path
{
    type Str: PathStr + ?Sized;

    const CURRENT_DIR: &'static Self::Str;
    const PARENT_DIR: &'static Self::Str;
    const SEPARATOR: &'static Self::Str;

    fn root() -> &'static Self;
    fn has_root(&self) -> bool;
    fn components(&self) -> Components<Self>;

    fn from_str(str: &Self::Str) -> &Self;

}
