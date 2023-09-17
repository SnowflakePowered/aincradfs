use std::borrow::Borrow;
use std::mem::ManuallyDrop;
use std::ops::{Deref, Index};
use std::slice::SliceIndex;
use bstr::{BStr, BString, ByteSlice, ByteVec};
use widestring::{U16Str, u16str, U16String};
use crate::path::components::{Component, Components};

mod u8path;
mod u16path;
mod components;

pub trait PathBuf {
    fn new() -> Self;
    fn push<A>(&mut self, entry: A)
        where A: Borrow<A>;

    fn pop(&mut self);
}

pub trait PathStr: 'static + PartialEq
{
    const CURRENT_DIR: &'static Self;
    const PARENT_DIR: &'static Self;
    const SEPARATOR: &'static Self;

    type ComponentType: Copy + PartialEq;

    fn is_separator(t: Self::ComponentType) -> bool;
    fn as_slice(&self) -> &[Self::ComponentType];
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn from_slice(slice: &[Self::ComponentType]) -> &Self;
}

impl PathStr for U16Str {
    const CURRENT_DIR: &'static U16Str = u16str!(".");
    const PARENT_DIR: &'static U16Str = u16str!("..");
    const SEPARATOR: &'static U16Str = u16str!("/");

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



#[inline]
const fn bstr_literal(x: &[u8]) -> &BStr {
    unsafe { core::mem::transmute(x) }
}

impl PathStr for BStr {
    const CURRENT_DIR: &'static BStr = bstr_literal(b".");
    const PARENT_DIR: &'static BStr = bstr_literal(b"..");
    const SEPARATOR: &'static BStr = bstr_literal(b"/");
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

pub trait PathOwned<Str: PathStr> {
    fn new() -> Self;
    fn push(&mut self, component: &Str);
    fn pop(&mut self);
}

pub trait Path<Str: PathStr + ?Sized>
{
    fn root() -> &'static Self;

    fn has_root(&self) -> bool;
    fn components(&self) -> Components<Str>;


}
