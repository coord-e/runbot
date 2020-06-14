use crate::model::compiler::Compiler;
use crate::model::language::LanguageName;
use crate::{Context, Error, Result};

pub fn list_compilers(ctx: &Context, language_name: LanguageName) -> Result<Vec<Compiler>> {
    let language = match ctx.config.find_language(&language_name) {
        Some(l) => l,
        None => return Err(Error::UnknownLanguageName(language_name)),
    };

    Ok(ctx
        .config
        .list_compilers_with_language_id(language.id())
        .cloned()
        .collect())
}
