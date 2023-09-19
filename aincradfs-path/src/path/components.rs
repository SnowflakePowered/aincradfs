use std::hash::Hash;
use std::iter::FusedIterator;

use crate::path::{Path, PathStr};

/// Component parsing works by a double-ended state machine; the cursors at the
/// front and back of the path each keep track of what parts of the path have
/// been consumed so far.
///
/// Going front to back, a path is made up of a prefix, a starting
/// directory component, and a body (of normal components)
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug)]
pub(crate) enum State {
    StartDir = 0, // / or . or nothing
    Body = 1,     // foo/bar/baz
    Done = 2,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Component<'a, P: Path + ?Sized> {
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
    Normal(&'a P::Str),
}
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Components<'a, P: Path + ?Sized> {
    // The path left to parse components from
    pub(crate) path: &'a P::Str,
    pub(crate) has_root: bool,
    // The iterator is double-ended, and these two states keep track of what has
    // been produced from either end
    pub(crate) front: State,
    pub(crate) back: State,
}

impl<'a, P: Path + ?Sized> Clone for Components<'a, P> {
    fn clone(&self) -> Self {
        Self {
            path: self.path,
            has_root: self.has_root,
            front: self.front.clone(),
            back: self.back.clone(),
        }
    }
}

impl<'a, P: Path + ?Sized> Components<'a, P> {
    // parse a given byte sequence following the OsStr encoding into the
    // corresponding path component
    fn parse_single_component(&self, comp: &'a P::Str) -> Option<Component<'a, P>> {
        // . components are normalized away, except at
        // the beginning of a path, which is treated
        // separately via `include_cur_dir`
        if comp == P::CURRENT_DIR {
            return None;
        };

        if comp == P::PARENT_DIR {
            return Some(Component::Parent);
        };

        if comp.is_empty() {
            return None;
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
        let root = if self.front <= State::StartDir && self.has_root {
            1
        } else {
            0
        };
        let cur_dir = if self.front <= State::StartDir && self.include_cur_dir() {
            1
        } else {
            0
        };
        root + cur_dir
    }

    // parse a component from the left, saying how many bytes to consume to
    // remove the component
    fn parse_next_component(&self) -> (usize, Option<Component<'a, P>>) {
        debug_assert!(self.front == State::Body);
        let (extra, comp) = match self
            .path
            .as_slice()
            .iter()
            .position(|b| P::is_separator(*b))
        {
            None => (0, self.path),
            Some(i) => (1, P::Str::from_slice(&self.path.as_slice()[..i])),
        };
        (comp.len() + extra, self.parse_single_component(comp))
    }

    // parse a component from the right, saying how many bytes to consume to
    // remove the component
    fn parse_next_component_back(&self) -> (usize, Option<Component<'a, P>>) {
        debug_assert!(self.back == State::Body);
        let start = self.len_before_body();
        let (extra, comp) = match self.path.as_slice()[start..]
            .iter()
            .rposition(|b| P::is_separator(*b))
        {
            None => (0, P::Str::from_slice(&self.path.as_slice()[start..])),
            Some(i) => (
                1,
                P::Str::from_slice(&self.path.as_slice()[start + i + 1..]),
            ),
        };
        (comp.len() + extra, self.parse_single_component(comp))
    }

    // trim away repeated separators (i.e., empty components) on the left
    fn trim_left(&mut self) {
        while !self.path.is_empty() {
            let (size, comp) = self.parse_next_component();
            if comp.is_some() {
                return;
            } else {
                self.path = P::Str::from_slice(&self.path.as_slice()[size..]);
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
                self.path = P::Str::from_slice(&self.path.as_slice()[..self.path.len() - size]);
            }
        }
    }

    /// Should the normalized path include a leading . ?
    fn include_cur_dir(&self) -> bool {
        if self.has_root {
            return false;
        }
        let mut iter = self.path.as_slice().iter();
        let current_dir = &P::Str::as_slice(P::CURRENT_DIR)[0];

        match (iter.next(), iter.next()) {
            (Some(c), None) if c == current_dir => true,
            (Some(c), Some(&b)) if c == current_dir => P::is_separator(b),
            _ => false,
        }
    }

    pub fn as_path(&self) -> &'a P {
        let mut comps = self.clone();
        if comps.front == State::Body {
            comps.trim_left();
        }
        if comps.back == State::Body {
            comps.trim_right();
        }
        P::from_str(comps.path)
    }
}

impl<'a, P: Path + ?Sized> Iterator for Components<'a, P> {
    type Item = Component<'a, P>;

    fn next(&mut self) -> Option<Component<'a, P>> {
        while !self.finished() {
            match self.front {
                State::StartDir => {
                    self.front = State::Body;
                    if self.has_root {
                        debug_assert!(!self.path.is_empty());
                        self.path = P::Str::from_slice(&self.path.as_slice()[1..]);
                        return Some(Component::Root);
                    } else if self.include_cur_dir() {
                        debug_assert!(!self.path.is_empty());
                        self.path = P::Str::from_slice(&self.path.as_slice()[1..]);
                        return Some(Component::Current);
                    }
                }
                State::Body if !self.path.is_empty() => {
                    let (size, comp) = self.parse_next_component();
                    self.path = P::Str::from_slice(&self.path.as_slice()[size..]);
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

impl<'a, P: Path + ?Sized> DoubleEndedIterator for Components<'a, P> {
    fn next_back(&mut self) -> Option<Component<'a, P>> {
        while !self.finished() {
            match self.back {
                State::Body if self.path.len() > self.len_before_body() => {
                    let (size, comp) = self.parse_next_component_back();
                    self.path = P::Str::from_slice(&self.path.as_slice()[..self.path.len() - size]);
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
                        self.path =
                            P::Str::from_slice(&self.path.as_slice()[..self.path.len() - 1]);
                        return Some(Component::Root);
                    } else if self.include_cur_dir() {
                        self.path =
                            P::Str::from_slice(&self.path.as_slice()[..self.path.len() - 1]);
                        return Some(Component::Current);
                    }
                }
                State::Done => unreachable!(),
            }
        }
        None
    }
}

impl<'a, P: Path + ?Sized> FusedIterator for Components<'a, P> {}

impl<'a, P: Path + ?Sized> PartialEq for Components<'a, P> {
    #[inline]
    fn eq(&self, other: &Components<'a, P>) -> bool {
        // Fast path for exact matches, e.g. for hashmap lookups.
        // Don't explicitly compare the prefix or has_physical_root fields since they'll
        // either be covered by the `path` buffer or are only relevant for `prefix_verbatim()`.
        if self.path.len() == other.path.len()
            && self.front == other.front
            && self.back == State::Body
            && other.back == State::Body
        {
            // possible future improvement: this could bail out earlier if there were a
            // reverse memcmp/bcmp comparing back to front
            if self.path == other.path {
                return true;
            }
        }

        let self_c = Components::clone(self);
        let other_c = Components::clone(other);

        // compare back to front since absolute paths often share long prefixes
        Iterator::eq(self_c.rev(), other_c.rev())
    }
}

impl<'a, P: Path + ?Sized> Eq for Components<'_, P> {}

#[cfg(test)]
mod test {
    use crate::path::u16path::U16PathBuf;
    use crate::path::u8path::U8PathBuf;
    use crate::path::Path;
    //
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

    #[test]
    pub fn test_eq() {
        let path = U16PathBuf::from("./test/my/./help//");
        let path2 = U16PathBuf::from("./test/my/help/");

        assert_eq!(path, path2)
    }

    #[test]
    pub fn test_eq_pathsep() {
        let path = U16PathBuf::from("./test/my\\help/");
        let path2 = U16PathBuf::from("./test/my/help/");

        assert_eq!(path, path2)
    }
}
