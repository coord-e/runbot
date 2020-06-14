use crate::model::language::Language;
use crate::Context;

pub fn list_languages(ctx: &Context) -> Vec<Language> {
    ctx.config.list_languages().cloned().collect()
}
