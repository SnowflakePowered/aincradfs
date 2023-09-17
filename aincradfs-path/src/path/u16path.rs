use std::borrow::Borrow;
use std::ops::Deref;
use widestring::{U16Str, u16str, U16String};
use crate::path::components::{Components, State};
use crate::path::{Path, PathOwned, PathStr};

#[repr(transparent)]
pub struct U16PathBuf(U16String);

#[repr(transparent)]
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

impl Path<U16Str> for U16Path {
    fn root() -> &'static Self {
        unsafe {
            std::mem::transmute(u16str!("/"))
        }
    }

    fn has_root(&self) -> bool {
        U16Str::is_separator(self.0.as_slice()[0])
    }

    fn components(&self) -> Components<U16Str> {
        Components {
            path: &self.0,
            has_root: self.has_root(),
            front: State::StartDir,
            back: State::Body,
        }
    }
}