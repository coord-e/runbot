use crate::model::compiler::CompilerName;
use crate::model::language::LanguageName;
use crate::{Context, Result};

pub struct Output {
    pub auto: bool,
    pub auto_save: bool,
    pub remap: Vec<(LanguageName, CompilerName)>,
}

pub fn dump_setting(ctx: &Context) -> Result<Output> {
    let auto = ctx.is_auto()?;
    let auto_save = ctx.is_auto_save()?;
    let remap = ctx
        .all_remap()?
        .into_iter()
        .map(|(l, c)| (l.clone(), c.clone()))
        .collect();

    Ok(Output {
        auto,
        auto_save,
        remap,
    })
}
