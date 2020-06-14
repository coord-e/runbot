use crate::setting::Scope;
use crate::{Context, Result};

pub fn set_auto_save(ctx: &Context, is_global: bool, set: bool) -> Result<()> {
    let scope = if is_global {
        Scope::Guild
    } else {
        Scope::Channel(ctx.channel_id)
    };

    ctx.setting.set_auto_save(ctx.guild_id, scope, set)?;

    Ok(())
}
