#![feature(unwrap_infallible)]

use std::sync::Arc;
use std::{env, result};

use runbot::action;
use runbot::model::channel_id::ChannelID;
use runbot::model::guild_id::GuildID;

use runbot_discord::code_input::CodeInput;
use runbot_discord::command_context::CommandContext;
use runbot_discord::error::{Error, Result};
use runbot_discord::table_loader;

use itertools::Itertools;
use parking_lot::Mutex;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::prelude::*;

pub struct ConnectionKey;

impl TypeMapKey for ConnectionKey {
    type Value = Arc<Mutex<redis::Connection>>;
}

struct RunbotHandler {
    table: runbot::Table,
    wandbox_client: wandbox::blocking::Client,
}

impl RunbotHandler {
    fn command_help(&self, ctx: &CommandContext) -> Result<()> {
        ctx.say(
            "
`!runbot` のあとに `global` をつけると全チャンネルに対して設定ができるよ
```
!runbot help            -- これ
!runbot show-setting    -- 設定表示
!runbot auto            -- 自動実行設定
!runbot no-auto         -- 自動実行解除
!runbot auto-save       -- 自動実行時に保存する
!runbot no-auto-save    -- 自動実行時に保存しない
!runbot remap           -- 言語名とコンパイラの紐付けを上書き
!runbot list-languages  -- 言語を一覧
!runbot list            -- 言語に対応するコンパイラを一覧
!runbot run             -- 実行
!runbot run-save        -- 実行して保存
```
",
        )
    }

    fn command_show_setting(&self, ctx: &CommandContext) -> Result<()> {
        let result = action::dump_setting(ctx)?;
        ctx.display_in_code_block(&result)
    }

    fn command_auto(&self, ctx: &CommandContext, state: bool) -> Result<()> {
        action::set_auto(ctx, ctx.is_global, state)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_auto_save(&self, ctx: &CommandContext, state: bool) -> Result<()> {
        action::set_auto_save(ctx, ctx.is_global, state)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_remap(&self, ctx: &CommandContext, commandline: &[impl AsRef<str>]) -> Result<()> {
        let (lang, compiler) = match commandline {
            [lang, compiler] => (
                lang.as_ref().parse().into_ok(),
                compiler.as_ref().parse().into_ok(),
            ),
            _ => return Err(Error::InvalidNumberOfArguments(2)),
        };

        action::remap_language(ctx, ctx.is_global, lang, compiler)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_list_languages(&self, ctx: &CommandContext) -> Result<()> {
        let languages = action::list_languages(ctx);
        ctx.display_in_code_block(&languages)
    }

    fn command_list(&self, ctx: &CommandContext, commandline: &[impl AsRef<str>]) -> Result<()> {
        let language = match commandline {
            [x] => x.as_ref().parse().into_ok(),
            _ => return Err(Error::InvalidNumberOfArguments(1)),
        };

        let compilers = action::list_compilers(ctx, language)?;
        ctx.display_in_code_block(&compilers)
    }

    fn command_run(
        &self,
        ctx: &CommandContext,
        commandline: &[impl AsRef<str>],
        body: &str,
        save: bool,
    ) -> Result<()> {
        let input: CodeInput = body.parse()?;

        let (compiler_spec, options) = match commandline.split_first() {
            Some((spec, [])) => (Some(spec.as_ref().parse().into_ok()), None),
            Some((spec, opts)) => (
                Some(spec.as_ref().parse().into_ok()),
                Some(opts.iter().map(|o| o.as_ref().to_string()).collect()),
            ),
            None => (None, None),
        };

        let result = action::run(
            ctx,
            compiler_spec,
            input.clone().into_code(),
            options,
            input.stdin().cloned(),
            save,
        )?;

        ctx.display(&result)
    }

    fn handle_implicit(&self, ctx: &CommandContext, content: &str) -> Result<()> {
        let input: CodeInput = match content.parse() {
            Err(_) => return Ok(()),
            Ok(x) => x,
        };

        let result = action::run_implicit(ctx, input.clone().into_code(), input.stdin().cloned())?;

        use action::run_implicit::Output;
        match result {
            Output::NoRun => Ok(()),
            Output::Run { .. } => {
                ctx.react(ReactionType::Unicode("🆗".to_string()))?;
                ctx.display(&result)
            }
        }
    }

    fn handle_explicit(&self, ctx: &mut CommandContext, line: &str, body: &str) -> Result<()> {
        ctx.react(ReactionType::Unicode("👀".to_string()))?;

        let line = if let Some(rest) = line.trim().strip_prefix("global") {
            ctx.is_global = true;
            rest
        } else {
            ctx.is_global = false;
            line
        };

        let words = shell_words::split(line)?;

        let (command, commandline) = match words.split_first() {
            Some(x) => x,
            None => return Err(Error::CommandIsMissing),
        };

        match command.as_ref() {
            "help" => self.command_help(ctx),
            "show-setting" => self.command_show_setting(ctx),
            "auto" => self.command_auto(ctx, true),
            "no-auto" => self.command_auto(ctx, false),
            "auto-save" => self.command_auto_save(ctx, true),
            "no-auto-save" => self.command_auto_save(ctx, false),
            "remap" => self.command_remap(ctx, commandline),
            "list-languages" => self.command_list_languages(ctx),
            "list" => self.command_list(ctx, commandline),
            "run" => self.command_run(ctx, commandline, body, false),
            "run-save" => self.command_run(ctx, commandline, body, true),
            _ => Err(Error::UnknownCommand(command.to_string())),
        }
    }

    fn handle(&self, ctx: &mut CommandContext, content: &str) -> Result<()> {
        let mut lines = content.lines();
        let line = lines.next().unwrap();
        let body = lines.join("\n");

        if let Some(rest) = line.strip_prefix("!runbot") {
            self.handle_explicit(ctx, rest, &body)
        } else {
            self.handle_implicit(ctx, content)
        }
    }
}

impl EventHandler for RunbotHandler {
    fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let guild_id = match msg.guild_id {
            Some(id) => GuildID::from_u64(*id.as_u64()),
            None => return,
        };
        let channel_id = ChannelID::from_u64(*msg.channel_id.as_u64());
        let msg_content = msg.content.clone();

        let redis_conn = ctx.data.read().get::<ConnectionKey>().unwrap().clone();
        let runbot_ctx = runbot::Context::new(
            guild_id,
            channel_id,
            self.wandbox_client.clone(),
            redis_conn,
            self.table.clone(),
        );

        let mut command_ctx = CommandContext::new(ctx, msg, runbot_ctx);
        if let Err(e) = self.handle(&mut command_ctx, &msg_content) {
            let _ = command_ctx.display(&e);
            eprintln!("command returned an error: {}", e);
        }
    }
}

fn main() -> result::Result<(), Box<dyn std::error::Error>> {
    let token = env::var("DISCORD_TOKEN")?;
    let redis_uri = env::var("REDIS_URI")?;
    let table_path = env::var("TABLE_FILE_PATH")?;
    let wandbox_home = env::var("WANDBOX_HOME")?;

    let table = table_loader::load_table(table_path)?;
    let wandbox_client = wandbox::blocking::Client::new(&wandbox_home)?;

    let mut client = Client::new(
        &token,
        RunbotHandler {
            table,
            wandbox_client,
        },
    )?;

    let redis_client = redis::Client::open(redis_uri)?;
    let redis_conn = Arc::new(Mutex::new(redis_client.get_connection()?));

    {
        let mut data = client.data.write();
        data.insert::<ConnectionKey>(redis_conn);
    }

    client.start()?;

    Ok(())
}
