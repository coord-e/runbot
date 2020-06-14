use std::{fmt, ops, str};

use super::display::Display;
use super::error::Result;

use itertools::Itertools;
use serenity::client::Context;
use serenity::model::channel::{Message, ReactionType};

pub struct CommandContext {
    pub ctx: Context,
    pub message: Message,
    pub is_global: bool,
    pub runbot_ctx: runbot::Context,
}

impl ops::Deref for CommandContext {
    type Target = runbot::Context;
    fn deref(&self) -> &runbot::Context {
        &self.runbot_ctx
    }
}

impl CommandContext {
    pub fn new(ctx: Context, message: Message, runbot_ctx: runbot::Context) -> CommandContext {
        CommandContext {
            ctx,
            message,
            is_global: false,
            runbot_ctx,
        }
    }

    pub fn say(&self, message: impl AsRef<str>) -> Result<()> {
        for msg in message.as_ref().chars().chunks(2000).into_iter() {
            let msg_str: String = msg.collect();
            self.message.channel_id.say(&self.ctx.http, &msg_str)?;
        }
        Ok(())
    }

    pub fn react(&self, reaction: ReactionType) -> Result<()> {
        self.message.react(&self.ctx.http, reaction)?;
        Ok(())
    }

    pub fn display<'a, T>(&self, x: &'a T) -> Result<()>
    where
        Display<'a, T>: fmt::Display,
    {
        self.say(Display(x).to_string())
    }

    pub fn display_in_code_block<'a, T>(&self, x: &'a T) -> Result<()>
    where
        Display<'a, T>: fmt::Display,
    {
        self.say(format!("```{}```", Display(x)))
    }
}
