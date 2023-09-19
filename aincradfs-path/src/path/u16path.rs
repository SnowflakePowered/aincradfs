use crate::path::components::{Components, State};
use crate::path::{Path, PathOwned, PathStr};
use qp_trie::Break;
use std::borrow::Borrow;
use std::ops::Deref;
use widestring::{u16str, U16Str, U16String};

#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct U16PathBuf(U16String);

#[repr(transparent)]
#[derive(Debug)]
pub struct U16Path(U16Str);

impl From<&U16Path> for U16PathBuf {
    fn from(value: &U16Path) -> Self {
        Self(value.0.to_ustring())
    }
}

impl From<&str> for U16PathBuf {
    fn from(value: &str) -> Self {
        Self(U16String::from_str(value))
    }
}

impl Borrow<U16Path> for U16PathBuf {
    fn borrow(&self) -> &U16Path {
        unsafe {
            // SAFETY: U16Path and U16Str have the same layout because repr(transparent).
            std::mem::transmute::<&U16Str, _>(self.0.borrow())
        }
    }
}

impl Deref for U16PathBuf {
    type Target = U16Path;

    fn deref(&self) -> &Self::Target {
        self.borrow()
    }
}

impl AsRef<U16Path> for U16PathBuf {
    fn as_ref(&self) -> &U16Path {
        self.borrow()
    }
}
impl ToOwned for U16Path {
    type Owned = U16PathBuf;

    fn to_owned(&self) -> Self::Owned {
        U16PathBuf::from(self)
    }
}

impl PartialEq for U16Path {
    fn eq(&self, other: &Self) -> bool {
        // fast path for exact match
        if self.0 == other.0 {
            return true;
        }

        self.components() == other.components()
    }
}

impl Eq for U16Path {}

impl PartialEq for U16PathBuf {
    fn eq(&self, other: &Self) -> bool {
        // let own_components = self.components().collect::<smallvec::SmallVec<>>();
        self.as_ref() == other.as_ref()
    }
}

impl Eq for U16PathBuf {}

impl Path for U16Path {
    type Str = U16Str;

    const CURRENT_DIR: &'static U16Str = u16str!(".");
    const PARENT_DIR: &'static U16Str = u16str!("..");
    const SEPARATOR: &'static U16Str = u16str!("/");

    fn is_separator(t: <Self::Str as PathStr>::ComponentType) -> bool {
        [b'/' as u16, b'\\' as u16].contains(&t)
    }

    fn root() -> &'static Self {
        unsafe { std::mem::transmute(Self::SEPARATOR) }
    }

    fn empty() -> &'static Self {
        const EMPTY: &'static U16Str = u16str!("");
        unsafe { std::mem::transmute(EMPTY) }
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
        // SAFETY: U16Path is repr(transparent) with U16Str
        unsafe { std::mem::transmute(str) }
    }
}

impl Borrow<[u8]> for U16Path {
    fn borrow(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0.as_slice())
    }
}

impl Borrow<[u8]> for U16PathBuf {
    fn borrow(&self) -> &[u8] {
        bytemuck::cast_slice(&self.0.as_slice())
    }
}

impl Break for U16PathBuf {
    type Split = U16Path;

    fn empty<'a>() -> &'a Self::Split {
        U16Path::empty()
    }

    fn find_break(&self, loc: usize) -> &Self::Split {
        let mut half_loc = loc / 2;

        while !U16Path::is_separator(self.0.as_slice()[half_loc]) {
            half_loc -= 1;
        }

        let slice = &self.0.as_slice()[..half_loc];
        unsafe { std::mem::transmute(slice) }
    }
}

impl PathOwned for U16PathBuf {
    type Borrowed = U16Path;
    fn new() -> Self {
        Self(U16String::new())
    }

    fn push(&mut self, _component: &<Self::Borrowed as Path>::Str) {
        todo!()
    }

    fn pop(&mut self) {
        todo!()
    }
}
