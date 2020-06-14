#![feature(str_strip)]
#![feature(unwrap_infallible)]

use std::sync::Arc;
use std::{env, result};

use runbot::action;
use runbot::model::channel_id::ChannelID;
use runbot::model::guild_id::GuildID;

use runbot_discord::code_input::CodeInput;
use runbot_discord::command_context::CommandContext;
use runbot_discord::error::{Error, Result};
use runbot_discord::table::Table;
use runbot_discord::{config_file, type_key};

use itertools::Itertools;
use parking_lot::Mutex;
use serenity::model::channel::Message;
use serenity::model::channel::ReactionType;
use serenity::prelude::*;

struct RunbotHandler(runbot::Config);

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
        let result = action::dump_setting(&ctx.action_ctx)?;
        ctx.print_code_block(Table(result).to_string())
    }

    fn command_auto(&self, ctx: &CommandContext, state: bool) -> Result<()> {
        action::set_auto(&ctx.action_ctx, ctx.is_global, state)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_auto_save(&self, ctx: &CommandContext, state: bool) -> Result<()> {
        action::set_auto_save(&ctx.action_ctx, ctx.is_global, state)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_remap(&self, ctx: &CommandContext, commandline: &[impl AsRef<str>]) -> Result<()> {
        let (lang, compiler) = match commandline {
            [lang, compiler] => (
                lang.as_ref().parse().into_ok(),
                compiler.as_ref().parse().into_ok(),
            ),
            _ => return ctx.unhandled("引数をちゃんと指定してね"),
        };

        action::remap_language(&ctx.action_ctx, ctx.is_global, lang, compiler)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_list_languages(&self, ctx: &CommandContext) -> Result<()> {
        let languages = action::list_languages(&ctx.action_ctx);
        ctx.print_code_block(Table(languages).to_string())
    }

    fn command_list(&self, ctx: &CommandContext, commandline: &[impl AsRef<str>]) -> Result<()> {
        let language = match commandline {
            [x] => x.as_ref().parse().into_ok(),
            _ => return ctx.unhandled("言語名をちゃんと指定してね"),
        };

        let compilers = action::list_compilers(&ctx.action_ctx, language)?;
        ctx.print_code_block(Table(compilers).to_string())
    }

    fn command_run(
        &self,
        ctx: &CommandContext,
        commandline: &[impl AsRef<str>],
        body: &str,
        save: bool,
    ) -> Result<()> {
        let input: CodeInput = match body.parse() {
            Err(_) => return ctx.unhandled("うーん"),
            Ok(x) => x,
        };

        let (compiler_spec, options) = match commandline.split_first() {
            Some((spec, [])) => (Some(spec.as_ref().parse().into_ok()), None),
            Some((spec, opts)) => (
                Some(spec.as_ref().parse().into_ok()),
                Some(opts.iter().map(|o| o.as_ref().to_string()).collect()),
            ),
            None => (None, None),
        };

        let result = action::run(
            &ctx.action_ctx,
            compiler_spec,
            input.clone().into_code(),
            options,
            input.stdin().cloned(),
            save,
        )?;

        ctx.print_compile_result(result)
    }

    fn handle_implicit(&self, ctx: &CommandContext, content: &str) -> Result<()> {
        let input: CodeInput = match content.parse() {
            Err(_) => return Ok(()),
            Ok(x) => x,
        };

        let result = action::run_implicit(
            &ctx.action_ctx,
            input.clone().into_code(),
            input.stdin().cloned(),
        )?;

        use action::run_implicit::Output;
        match result {
            Output::NoRun => Ok(()),
            Output::Run { .. } => {
                ctx.react(ReactionType::Unicode("🆗".to_string()))?;
                ctx.print_compile_result(result)
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

        let words = match shell_words::split(line) {
            Ok(x) => x,
            Err(_) => return ctx.unhandled("うーん？"),
        };

        let (command, commandline) = match words.split_first() {
            Some(x) => x,
            None => return ctx.unhandled("引数が足りないにえ"),
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
            _ => ctx.unhandled("知らないコマンドニャンね"),
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

        let mut command_ctx = CommandContext::new(ctx, msg, guild_id, channel_id, self.0.clone());
        if let Err(e) = self.handle(&mut command_ctx, &msg_content) {
            let _ = match &e {
                Error::Runbot(runbot::Error::UnknownLanguageName(name)) => {
                    command_ctx.say(format!("`{}` ？うーん...", name))
                }
                Error::Runbot(runbot::Error::UnknownCompilerName(name)) => {
                    command_ctx.say(format!("`{}` っ て 何 ？ 笑", name))
                }
                Error::Runbot(runbot::Error::UnknownCompilerSpec(name)) => {
                    command_ctx.say(format!(
                        "`{}` とはなんですか？普通、`{}` とはならないとおもうのですが...",
                        name, name
                    ))
                }
                Error::Runbot(runbot::Error::UnmappedLanguage(name)) => {
                    command_ctx.say(format!("`{}` に対応するコンパイラが決まっていない", name))
                }
                Error::Runbot(runbot::Error::NoCompilerSpecified) => {
                    command_ctx.say("どのコンパイラを使えばいいかわかんないよ〜")
                }
                Error::Runbot(runbot::Error::RemapMismatch(c, l)) => {
                    command_ctx.say(format!("や、`{}` は `{}` でコンパイルできないよ", l, c))
                }
                _ => command_ctx.say("ごめん"),
            };
            eprintln!("command returned an error: {}", e);
        }
    }
}

fn main() -> result::Result<(), Box<dyn std::error::Error>> {
    let token = env::var("DISCORD_TOKEN")?;
    let redis_uri = env::var("REDIS_URI")?;
    let config_path = env::var("CONFIG_PATH")?;

    let config = config_file::load_config(config_path)?;
    let mut client = Client::new(&token, RunbotHandler(config))?;

    let redis_client = redis::Client::open(redis_uri)?;
    let redis_conn = Arc::new(Mutex::new(redis_client.get_connection()?));

    {
        let mut data = client.data.write();
        data.insert::<type_key::ConnectionKey>(redis_conn);
    }

    client.start()?;

    Ok(())
}
