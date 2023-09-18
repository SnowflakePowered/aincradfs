use std::borrow::Borrow;
use std::ops::Deref;
use qp_trie::Break;
use widestring::{U16Str, u16str, U16String};
use crate::path::components::{Components, State};
use crate::path::{Path, PathOwned, PathStr, U8PathBuf};

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

impl ToOwned for U16Path {
    type Owned = U16PathBuf;

    fn to_owned(&self) -> Self::Owned {
        U16PathBuf::from(self)
    }
}

impl Path for U16Path {
    type Str = U16Str;

    const CURRENT_DIR: &'static U16Str = u16str!(".");
    const PARENT_DIR: &'static U16Str = u16str!("..");
    const SEPARATOR: &'static U16Str = u16str!("/");

    fn root() -> &'static Self {
        unsafe {
            std::mem::transmute(u16str!("/"))
        }
    }

    fn has_root(&self) -> bool {
        U16Str::is_separator(self.0.as_slice()[0])
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

impl Borrow<<Self as Break>::Split> for U16PathBuf {
    fn borrow(&self) -> &<Self as Break>::Split {
        bytemuck::cast_slice(&self.0.as_slice())
    }
}

impl Break for U16PathBuf {
    type Split = [u8];

    fn empty<'a>() -> &'a Self::Split {
        <&'a [u8]>::default()
    }

    fn find_break(&self, loc: usize) -> &Self::Split {
        &<Self as Borrow<[u8]>>::borrow(self)[..loc]
    }
}

impl PathOwned for U16PathBuf {
    type Borrowed = U16Path;
    fn new() -> Self {
        Self(U16String::new())
    }

    fn push(&mut self, component: &<Self::Borrowed as Path>::Str) {
        todo!()
    }


    fn pop(&mut self) {
        todo!()
    }
}