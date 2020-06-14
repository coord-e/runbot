use std::collections::HashMap;

use crate::model::compiler::{Compiler, CompilerID, CompilerName};
use crate::model::language::{Language, LanguageID, LanguageName};

#[derive(Clone)]
pub struct Table {
    pub compilers: HashMap<CompilerID, Compiler>,
    pub languages: HashMap<LanguageID, Language>,
}

impl Table {
    pub fn get_compiler(&self, id: CompilerID) -> &Compiler {
        self.compilers.get(&id).expect("unknown compiler ID")
    }

    pub fn find_compiler(&self, name: &CompilerName) -> Option<&Compiler> {
        self.compilers.values().find(|c| c.name() == name)
    }

    pub fn list_compilers_with_language_id(
        &self,
        id: LanguageID,
    ) -> impl Iterator<Item = &Compiler> {
        self.compilers
            .values()
            .filter(move |c| c.language_id() == id)
    }

    pub fn get_language(&self, id: LanguageID) -> &Language {
        self.languages.get(&id).expect("unknown language ID")
    }

    pub fn find_language(&self, name: &LanguageName) -> Option<&Language> {
        self.languages.values().find(|c| c.is_named_as(name))
    }

    pub fn list_languages(&self) -> impl Iterator<Item = &Language> {
        self.languages.values()
    }
}
