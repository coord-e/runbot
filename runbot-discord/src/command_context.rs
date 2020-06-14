use std::io::Write;
use std::{ops, str};

use super::compile_result::CompileResult;
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

    pub fn unhandled(&self, message: impl AsRef<str>) -> Result<()> {
        self.say(format!("わからん、{}", message.as_ref()))
    }

    pub fn react(&self, reaction: ReactionType) -> Result<()> {
        self.message.react(&self.ctx.http, reaction)?;
        Ok(())
    }

    pub fn print_code_block(&self, s: impl AsRef<str>) -> Result<()> {
        self.say(format!("```{}```", s.as_ref()))
    }

    pub fn print_compile_result<R>(&self, res: R) -> Result<()>
    where
        R: CompileResult,
    {
        let mut buf = strip_ansi_escapes::Writer::new(Vec::new());

        if let Some(msg) = res.compiler_message() {
            writeln!(buf, "```{}```", msg)?;
        }

        if let Some(msg) = res.program_message() {
            writeln!(buf, "```{}```", msg)?;
        }

        if let Some(s) = res.signal() {
            writeln!(buf, "exited with signal: {}", s)?;
        }

        if let Some(s) = res.status() {
            writeln!(buf, "exited with status code {}", s)?;
        }

        if let Some(s) = res.url() {
            writeln!(buf, "{}", s)?;
        }

        let msg = buf.into_inner()?;
        self.say(str::from_utf8(&msg)?)
    }
}
