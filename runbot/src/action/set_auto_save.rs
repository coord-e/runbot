use crate::setting::Scope;
use crate::{ActionContext, Result};

pub fn set_auto_save(ctx: &ActionContext, is_global: bool, set: bool) -> Result<()> {
    let scope = if is_global {
        Scope::Guild
    } else {
        Scope::Channel(ctx.channel_id())
    };

    ctx.setting.set_auto_save(ctx.guild_id, scope, set)?;

    Ok(())
}
