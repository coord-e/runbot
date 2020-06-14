use crate::setting::Scope;
use crate::{Context, Result};

pub fn set_auto(ctx: &Context, is_global: bool, set: bool) -> Result<()> {
    let scope = if is_global {
        Scope::Guild
    } else {
        Scope::Channel(ctx.channel_id)
    };

    ctx.setting.set_auto(ctx.guild_id, scope, set)?;

    Ok(())
}
