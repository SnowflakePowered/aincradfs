use std::cmp;
use std::hash::{Hash, Hasher};
use std::ops::Index;
use std::slice::SliceIndex;
use crate::path::PathStr;


/// Component parsing works by a double-ended state machine; the cursors at the
/// front and back of the path each keep track of what parts of the path have
/// been consumed so far.
///
/// Going front to back, a path is made up of a prefix, a starting
/// directory component, and a body (of normal components)
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub enum State {
    StartDir = 0, // / or . or nothing
    Body = 1,     // foo/bar/baz
    Done = 2,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Component<'a, Str: ?Sized> {
    /// The root directory component, appears after any prefix and before anything else.
    ///
    /// It represents a separator that designates that a path starts from root.
    Root,

    /// A reference to the current directory, i.e., `.`.
    Current,

    /// A reference to the parent directory, i.e., `..`.
    Parent,

    /// A normal component, e.g., `a` and `b` in `a/b`.
    ///
    /// This variant is the most common one, it represents references to files
    /// or directories.
    Normal(&'a Str),
}

#[derive(Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Components<'a, Str: PathStr + ?Sized> {
    // The path left to parse components from
    pub(crate) path: &'a Str,
    pub(crate) has_root: bool,
    // The iterator is double-ended, and these two states keep track of what has
    // been produced from either end
    pub(crate) front: State,
    pub(crate) back: State,
}

impl<'a, Str: PathStr + ?Sized> Components<'a, Str>
{
    // parse a given byte sequence following the OsStr encoding into the
    // corresponding path component
    fn parse_single_component(&self, comp: &'a Str) -> Option<Component<'a, Str>> {

        // . components are normalized away, except at
        // the beginning of a path, which is treated
        // separately via `include_cur_dir`
        if comp == Str::CURRENT_DIR {
            return None
        };

        if comp == Str::PARENT_DIR {
            return Some(Component::Parent)
        };

        if comp.is_empty() {
            return None
        }

        Some(Component::Normal(comp))
    }

    #[inline]
    fn finished(&self) -> bool {
        self.front == State::Done || self.back == State::Done || self.front > self.back
    }

    // Given the iteration so far, how much of the pre-State::Body path is left?
    #[inline]
    fn len_before_body(&self) -> usize {
        let root = if self.front <= State::StartDir && self.has_root { 1 } else { 0 };
        let cur_dir = if self.front <= State::StartDir && self.include_cur_dir() { 1 } else { 0 };
        root + cur_dir
    }

    // parse a component from the left, saying how many bytes to consume to
    // remove the component
    fn parse_next_component(&self) -> (usize, Option<Component<'a, Str>>) {
        debug_assert!(self.front == State::Body);
        let (extra, comp) = match self.path.as_slice().iter().position(|b| Str::is_separator(*b)) {
            None => (0, self.path),
            Some(i) => (1,  Str::from_slice(&self.path.as_slice()[..i])),
        };
        // SAFETY: `comp` is a valid substring, since it is split on a separator.
        (comp.len() + extra, unsafe { self.parse_single_component(comp) })
    }

    // parse a component from the right, saying how many bytes to consume to
    // remove the component
    fn parse_next_component_back(&self) -> (usize, Option<Component<'a, Str>>) {
        debug_assert!(self.back == State::Body);
        let start = self.len_before_body();
        let (extra, comp) = match self.path.as_slice()[start..].iter().rposition(|b| Str::is_separator(*b)) {
            None => (0, Str::from_slice(&self.path.as_slice()[start..])),
            Some(i) => (1, Str::from_slice(&self.path.as_slice()[start + i + 1..])),
        };
        // SAFETY: `comp` is a valid substring, since it is split on a separator.
        (comp.len() + extra, unsafe { self.parse_single_component(comp) })
    }

    // trim away repeated separators (i.e., empty components) on the left
    fn trim_left(&mut self) {
        while !self.path.is_empty() {
            let (size, comp) = self.parse_next_component();
            if comp.is_some() {
                return;
            } else {
                self.path = Str::from_slice(&self.path.as_slice()[size..]);
            }
        }
    }

    // trim away repeated separators (i.e., empty components) on the right
    fn trim_right(&mut self) {
        while self.path.len() > self.len_before_body() {
            let (size, comp) = self.parse_next_component_back();
            if comp.is_some() {
                return;
            } else {
                self.path = Str::from_slice(&self.path.as_slice()[..self.path.len() - size]);
            }
        }
    }


    /// Should the normalized path include a leading . ?
    fn include_cur_dir(&self) -> bool {
        if self.has_root {
            return false;
        }
        let mut iter = self.path.as_slice().iter();
        let current_dir = &Str::as_slice(Str::CURRENT_DIR)[0];

        match (iter.next(), iter.next()) {
            (Some(c), None) if c == current_dir => true,
            (Some(c), Some(&b)) if c == current_dir => Str::is_separator(b),
            _ => false,
        }
    }
}

impl<'a, Str: PathStr + ?Sized> Iterator for Components<'a, Str> {
    type Item = Component<'a, Str>;

    fn next(&mut self) -> Option<Component<'a, Str>> {
        while !self.finished() {
            match self.front {
                State::StartDir => {
                    self.front = State::Body;
                    if self.has_root {
                        debug_assert!(!self.path.is_empty());
                        self.path = Str::from_slice(&self.path.as_slice()[1..]);
                        return Some(Component::Root);
                    } else if self.include_cur_dir() {
                        debug_assert!(!self.path.is_empty());
                        self.path = Str::from_slice(&self.path.as_slice()[1..]);
                        return Some(Component::Current);
                    }
                }
                State::Body if !self.path.is_empty() => {
                    let (size, comp) = self.parse_next_component();
                    self.path = Str::from_slice(&self.path.as_slice()[size..]);
                    if comp.is_some() {
                        return comp;
                    }
                }
                State::Body => {
                    self.front = State::Done;
                }
                State::Done => unreachable!(),
            }
        }
        None
    }
}

impl<'a, Str: PathStr> DoubleEndedIterator for Components<'a, Str> {
    fn next_back(&mut self) -> Option<Component<'a, Str>> {
        while !self.finished() {
            match self.back {
                State::Body if self.path.len() > self.len_before_body() => {
                    let (size, comp) = self.parse_next_component_back();
                    self.path = Str::from_slice(&self.path.as_slice()[..self.path.len() - size]);
                    if comp.is_some() {
                        return comp;
                    }
                }
                State::Body => {
                    self.back = State::StartDir;
                }
                State::StartDir => {
                    self.back = State::Done;
                    if self.has_root {
                        self.path = Str::from_slice(&self.path.as_slice()[..self.path.len() - 1]);
                        return Some(Component::Root);
                    } else if self.include_cur_dir() {
                        self.path = Str::from_slice(&self.path.as_slice()[..self.path.len() - 1]);
                        return Some(Component::Current);
                    }
                }
                State::Done => unreachable!(),
            }
        }
        None
    }
}

#[cfg(test)]
mod test {
    use crate::path::Path;
    use crate::path::u16path::{ U16PathBuf};
    use crate::path::u8path::U8PathBuf;

    #[test]
    pub fn test_wstr() {
        let path = U16PathBuf::from("./test/my/./../help//");

        for component in path.components() {
            println!("{:?}", component);
        }
    }

    #[test]
    pub fn test_u8str() {
        let path = U8PathBuf::from("./test/my/./../help//");

        for component in path.components() {
            println!("{:?}", component);
        }
    }
}