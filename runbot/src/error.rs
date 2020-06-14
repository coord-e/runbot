use std::result;

use crate::model::compiler::CompilerName;
use crate::model::compiler_spec::CompilerSpec;
use crate::model::language::LanguageName;

use err_derive::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "unknown language name {}", _0)]
    UnknownLanguageName(LanguageName),
    #[error(display = "unknown compiler name {}", _0)]
    UnknownCompilerName(CompilerName),
    #[error(display = "unknown compiler spec {}", _0)]
    UnknownCompilerSpec(CompilerSpec),
    #[error(display = "no (default) compiler can be found for language {}", _0)]
    UnmappedLanguage(LanguageName),
    #[error(display = "no compiler is specified, but is required")]
    NoCompilerSpecified,
    #[error(display = "{} is not a compiler for {}", _0, _1)]
    RemapMismatch(CompilerName, LanguageName),
    #[error(display = "network error: {}", _0)]
    Wandbox(#[error(source)] wandbox::Error),
    #[error(display = "database error: {}", _0)]
    Database(#[error(source)] redis::RedisError),
}

pub type Result<T> = result::Result<T, Error>;
