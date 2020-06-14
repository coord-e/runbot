use std::str::FromStr;

use crate::model::compiler::CompilerName;
use crate::model::language::LanguageName;

use derive_more::Display;

#[derive(Debug, Clone, Display)]
pub struct CompilerSpec(String);

impl FromStr for CompilerSpec {
    type Err = !;
    fn from_str(s: &str) -> Result<CompilerSpec, !> {
        Ok(CompilerSpec(s.to_owned()))
    }
}

impl CompilerSpec {
    pub fn as_language_name(&self) -> &LanguageName {
        LanguageName::from_string_ref(&self.0)
    }

    pub fn as_compiler_name(&self) -> &CompilerName {
        CompilerName::from_string_ref(&self.0)
    }
}
