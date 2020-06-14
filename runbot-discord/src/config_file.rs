use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::{fs, io};

use runbot::model::compiler::{Compiler, CompilerID, CompilerName, CompilerVersion};
use runbot::model::language::{Language, LanguageID, LanguageName};
use runbot::Config;

use err_derive::Error;
use serde::Deserialize;

#[derive(Deserialize)]
struct ConfigFile {
    languages: HashMap<String, LanguageData>,
}

#[derive(Deserialize)]
struct LanguageData {
    aliases: Vec<String>,
    compilers: HashMap<String, CompilerData>,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CompilerData {
    version: Option<String>,
    wandbox_name: String,
    default: Option<bool>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Duplicated default compiler found for language {}", _0)]
    DuplicateDefault(LanguageName),
    #[error(display = "IO error: {}", _0)]
    IO(#[error(cause)] io::Error),
}

pub fn load_config(path: impl AsRef<Path>) -> Result<Config, Error> {
    let content = fs::read(path)?;
    let config_file = toml::from_slice(&content).unwrap();
    to_config(config_file)
}

fn to_config(config_file: ConfigFile) -> Result<Config, Error> {
    let mut languages = HashMap::new();
    let mut compilers = HashMap::new();

    for (language_name, language_data) in config_file.languages.into_iter() {
        let language_id = LanguageID::from_u64(hash_to_u64(&language_name));
        let language_name = LanguageName::from_string(language_name);

        let mut default_compiler = None;
        for (compiler_name, compiler_data) in language_data.compilers.into_iter() {
            let compiler_id = CompilerID::from_u64(hash_to_u64(&compiler_name));
            let compiler_name = CompilerName::from_string(compiler_name);
            let compiler_version = compiler_data.version.map(CompilerVersion::from_string);

            if compiler_data.default.unwrap_or(false) {
                if default_compiler.is_some() {
                    return Err(Error::DuplicateDefault(language_name));
                }
                default_compiler = Some(compiler_id);
            }

            let compiler = Compiler::new(
                compiler_id,
                compiler_name,
                compiler_version,
                language_id,
                compiler_data.wandbox_name,
            );
            compilers.insert(compiler_id, compiler);
        }

        let names = language_data
            .aliases
            .into_iter()
            .map(LanguageName::from_string)
            .collect();
        let language = Language::new(language_id, language_name, names, default_compiler);
        languages.insert(language_id, language);
    }

    Ok(Config {
        languages,
        compilers,
    })
}

fn hash_to_u64<T: Hash>(x: &T) -> u64 {
    use rustc_hash::FxHasher;
    let mut hasher = FxHasher::default();
    x.hash(&mut hasher);
    hasher.finish()
}
