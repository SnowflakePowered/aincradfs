use crate::path::components::{Components, State};
use crate::path::{Path, PathOwned, PathStr};
use bstr::{BStr, BString};
use qp_trie::Break;
use std::borrow::Borrow;
use std::ops::Deref;

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

impl AsRef<U8Path> for U8PathBuf {
    fn as_ref(&self) -> &U8Path {
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

impl PartialEq for U8Path {
    fn eq(&self, other: &Self) -> bool {
        // let own_components = self.components().collect::<smallvec::SmallVec<>>();
        self.components() == other.components()
    }
}

impl Eq for U8Path {}

impl PartialEq for U8PathBuf {
    fn eq(&self, other: &Self) -> bool {
        // fast path for exact match
        if self.0 == other.0 {
            return true;
        }

        self.as_ref() == other.as_ref()
    }
}

impl Eq for U8PathBuf {}

impl Path for U8Path {
    type Str = BStr;

    const CURRENT_DIR: &'static BStr = bstr_literal(b".");
    const PARENT_DIR: &'static BStr = bstr_literal(b"..");
    const SEPARATOR: &'static BStr = bstr_literal(b"/");

    fn is_separator(t: <Self::Str as PathStr>::ComponentType) -> bool {
        [b'/', b'\\'].contains(&t)
    }

    fn root() -> &'static Self {
        unsafe {
            // SAFETY: U8Path and BStr have the same layout because repr(transparent).
            std::mem::transmute::<&BStr, _>(Self::SEPARATOR)
        }
    }

    fn empty() -> &'static Self {
        const EMPTY: &'static BStr = bstr_literal(b"");
        unsafe {
            // SAFETY: U8Path and BStr have the same layout because repr(transparent).
            std::mem::transmute::<&BStr, _>(EMPTY)
        }
    }

    fn has_root(&self) -> bool {
        Self::is_separator(self.0.as_slice()[0])
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

impl Borrow<[u8]> for U8PathBuf {
    fn borrow(&self) -> &[u8] {
        &self.0.as_slice()
    }
}

impl Break for U8PathBuf {
    type Split = U8Path;

    fn empty<'a>() -> &'a Self::Split {
        U8Path::empty()
    }

    fn find_break(&self, mut loc: usize) -> &Self::Split {
        while !U8Path::is_separator(self.0[loc]) {
            loc -= 1;
        }

        // SAFETY: BStr has the same layout as [] as U8Path
        unsafe { std::mem::transmute(&self.0.as_slice()[..loc]) }
    }
}

impl PathOwned for U8PathBuf {
    type Borrowed = U8Path;

    fn new() -> Self {
        Self(BString::new(Vec::new()))
    }

    fn push(&mut self, _component: &<Self::Borrowed as Path>::Str) {
        todo!()
    }

    fn pop(&mut self) {
        todo!()
    }
}
