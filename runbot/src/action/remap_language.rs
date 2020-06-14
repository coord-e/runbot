use crate::model::compiler::CompilerName;
use crate::model::language::LanguageName;
use crate::setting::Scope;
use crate::{Context, Error, Result};

pub fn remap_language(
    ctx: &Context,
    is_global: bool,
    language_name: LanguageName,
    compiler_name: CompilerName,
) -> Result<()> {
    let language = match ctx.table.find_language(&language_name) {
        Some(l) => l,
        None => return Err(Error::UnknownLanguageName(language_name)),
    };

    let compiler = match ctx.table.find_compiler(&compiler_name) {
        Some(c) => c,
        None => return Err(Error::UnknownCompilerName(compiler_name)),
    };

    if compiler.language_id() != language.id() {
        return Err(Error::RemapMismatch(
            compiler.name().clone(),
            language.name().clone(),
        ));
    }

    let scope = if is_global {
        Scope::Guild
    } else {
        Scope::Channel(ctx.channel_id)
    };

    ctx.setting
        .set_remap(ctx.guild_id, scope, language.id(), compiler.id())?;

    Ok(())
}
