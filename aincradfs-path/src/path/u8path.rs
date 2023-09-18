use std::borrow::Borrow;
use std::ops::Deref;
use bstr::{BStr, BString};
use qp_trie::Break;
use crate::path::components::{Components, State};
use crate::path::{Path, PathOwned, PathStr};

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct U8PathBuf(BString);

#[repr(transparent)]
#[derive(Debug)]
pub struct U8Path(BStr);


impl From<&U8Path> for U8PathBuf {
    fn from(value: &U8Path) -> Self {
        Self(BString::new(value.0.to_vec()))
    }
}

impl From<&str> for U8PathBuf {
    fn from(value: &str) -> Self {
        Self(BString::from(value))
    }
}

impl Borrow<U8Path> for U8PathBuf {
    fn borrow(&self) -> &U8Path {
        unsafe {
            // SAFETY: U8Path and BStr have the same layout because repr(transparent).
            std::mem::transmute::<&BStr, _>(self.0.borrow())
        }
    }
}

impl Deref for U8PathBuf {
    type Target = U8Path;

    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl ToOwned for U8Path {
    type Owned = U8PathBuf;

    fn to_owned(&self) -> Self::Owned {
        U8PathBuf::from(self)
    }
}

#[inline]
const fn bstr_literal(x: &[u8]) -> &BStr {
    unsafe { core::mem::transmute(x) }
}


impl Path for U8Path {
    type Str = BStr;

    const CURRENT_DIR: &'static BStr = bstr_literal(b".");
    const PARENT_DIR: &'static BStr = bstr_literal(b"..");
    const SEPARATOR: &'static BStr = bstr_literal(b"/");

    fn root() -> &'static Self {
        unsafe {
            // SAFETY: U8Path and BStr have the same layout because repr(transparent).
            std::mem::transmute::<&BStr, _>(BStr::new(b"/"))
        }
    }

    fn has_root(&self) -> bool {
        BStr::is_separator(self.0.as_slice()[0])
    }


    fn components(&self) -> Components<Self> {
        Components {
            path: &self.0,
            has_root: self.has_root(),
            front: State::StartDir,
            back: State::Body,
        }
    }

    fn from_str(str: &Self::Str) -> &Self {
        // SAFETY: U8Path is repr(transparent) with BStr
        unsafe { std::mem::transmute(str) }
    }
}

impl Borrow<[u8]> for U8Path {
    fn borrow(&self) -> &[u8] {
        &self.0.as_slice()
    }
}

impl Borrow<<Self as Break>::Split> for U8PathBuf {
    fn borrow(&self) -> &<Self as Break>::Split {
        &self.0.as_slice()
    }
}

impl Break for U8PathBuf {
    type Split = [u8];

    fn empty<'a>() -> &'a Self::Split {
        <&'a [u8]>::default()
    }

    fn find_break(&self, loc: usize) -> &Self::Split {
        &<Self as Borrow<[u8]>>::borrow(self)[..loc]
    }
}

impl PathOwned for U8PathBuf {
    type Borrowed = U8Path;

    fn new() -> Self {
        Self(BString::new(Vec::new()))
    }

    fn push(&mut self, component: &<Self::Borrowed as Path>::Str) {
        todo!()
    }

    fn pop(&mut self) {
        todo!()
    }
}