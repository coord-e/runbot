use crate::model::language::Language;
use crate::Context;

pub fn list_languages(ctx: &Context) -> Vec<Language> {
    ctx.table.list_languages().cloned().collect()
}
