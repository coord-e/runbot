use crate::model::language::Language;
use crate::ActionContext;

pub fn list_languages(ctx: &ActionContext) -> Vec<Language> {
    ctx.config.list_languages().cloned().collect()
}
