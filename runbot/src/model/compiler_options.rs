use std::{iter::FromIterator, slice, vec};

#[derive(Default, Debug, Clone)]
pub struct CompilerOptions(Vec<String>);

impl CompilerOptions {
    pub fn new() -> CompilerOptions {
        CompilerOptions::default()
    }
}

impl IntoIterator for CompilerOptions {
    type Item = String;
    type IntoIter = vec::IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a CompilerOptions {
    type Item = &'a String;
    type IntoIter = slice::Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<String> for CompilerOptions {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = String>,
    {
        CompilerOptions(iter.into_iter().collect())
    }
}
