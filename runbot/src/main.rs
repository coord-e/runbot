#![feature(str_strip)]
#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::{env, fmt, io, result, str};

use itertools::Itertools;
use maplit::{convert_args, hashmap};
use serenity::{
    model::{
        channel::{Message, ReactionType},
        gateway::Ready,
        id,
    },
    prelude::*,
};

struct CodeBlock {
    language: Option<String>,
    code: String,
}

impl str::FromStr for CodeBlock {
    type Err = !;
    fn from_str(s: &str) -> result::Result<CodeBlock, !> {
        let s = s.trim_matches('`');
        let split = s.splitn(2, '\n').collect::<Vec<_>>();
        Ok(match &split[..] {
            [head, rest] => {
                let head = head.trim();
                if head.is_empty() {
                    CodeBlock {
                        language: None,
                        code: rest.to_string(),
                    }
                } else {
                    CodeBlock {
                        language: Some(head.to_string()),
                        code: rest.to_string(),
                    }
                }
            }
            _ => CodeBlock {
                language: None,
                code: s.to_string(),
            },
        })
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
struct CompilerId {
    compiler_name: String,
}

impl str::FromStr for CompilerId {
    type Err = !;
    fn from_str(s: &str) -> result::Result<CompilerId, !> {
        Ok(CompilerId {
            compiler_name: s.to_owned(),
        })
    }
}

impl CompilerId {
    fn new(s: &str) -> CompilerId {
        s.parse().into_ok()
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum CompilerPredicate {
    NameSubstring(String),
}

impl str::FromStr for CompilerPredicate {
    type Err = !;
    fn from_str(s: &str) -> result::Result<CompilerPredicate, !> {
        Ok(CompilerPredicate::NameSubstring(s.to_owned()))
    }
}

impl CompilerPredicate {
    fn match_with(&self, input: &str) -> bool {
        match self {
            CompilerPredicate::NameSubstring(s) => input.contains(s),
        }
    }
}

#[derive(Default, Clone, Debug)]
struct Setting {
    auto: Option<bool>,
    auto_save: Option<bool>,
    language_map: HashMap<String, CompilerId>,
    default_compiler_options: HashMap<CompilerPredicate, Vec<String>>,
}

impl fmt::Display for Setting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl Setting {
    fn nice_language_map() -> Setting {
        let mut setting = Setting::default();
        setting.language_map = convert_args!(
            keys = String::from,
            values = CompilerId::new,
            hashmap!( "c" => "gcc-9.3.0-c"
            , "cpp" => "gcc-9.3.0"
            , "c++" => "gcc-9.3.0"
            , "cs" => "mono-5.8.0.108"
            , "python" => "cpython-3.8.0"
            , "rust" => "rust-1.18.0"
            , "js" => "nodejs-14.0.0"
            , "javascript" => "nodejs-14.0.0"
            , "d" => "dmd-2.076.0"
            , "go" => "go-1.14.2"
            , "haskell" => "ghc-8.4.2"
            , "bash" => "bash"
            )
        );
        setting
    }

    fn auto(&self) -> bool {
        self.auto.unwrap_or(true)
    }

    fn auto_save(&self) -> bool {
        self.auto_save.unwrap_or(false)
    }

    fn extend(&mut self, s: Setting) {
        self.auto = s.auto.or(self.auto);
        self.auto_save = s.auto_save.or(self.auto_save);
        self.language_map.extend(s.language_map);
        self.default_compiler_options
            .extend(s.default_compiler_options);
    }
}

#[derive(Debug, Clone)]
struct GuildSetting {
    inner: HashMap<id::ChannelId, Setting>,
    current_default: Setting,
}

impl fmt::Display for GuildSetting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl GuildSetting {
    fn new() -> GuildSetting {
        GuildSetting {
            inner: HashMap::new(),
            current_default: Setting::nice_language_map(),
        }
    }

    fn get_channel(&self, channel_id: &id::ChannelId) -> &Setting {
        self.inner.get(channel_id).unwrap_or(&self.current_default)
    }

    fn set_channel(&mut self, channel_id: &id::ChannelId, setting: Setting) {
        let mut s = self.get_channel(channel_id).clone();
        s.extend(setting);
        self.inner.insert(channel_id.clone(), s);
    }

    fn set_all(&mut self, s: Setting) {
        let channels = self.inner.keys().cloned().collect::<Vec<_>>();
        for channel_id in channels {
            self.set_channel(&channel_id, s.clone());
        }

        self.current_default.extend(s);
    }
}

#[derive(Debug, Clone)]
struct GlobalSetting {
    inner: HashMap<id::GuildId, GuildSetting>,
}

impl GlobalSetting {
    fn new() -> GlobalSetting {
        GlobalSetting {
            inner: HashMap::new(),
        }
    }

    fn get_mut_guild(&mut self, guild_id: &id::GuildId) -> &mut GuildSetting {
        self.inner
            .entry(guild_id.clone())
            .or_insert_with(|| GuildSetting::new())
    }
}

impl TypeMapKey for GlobalSetting {
    type Value = GlobalSetting;
}

#[derive(Debug)]
enum Error {
    Lock,
    Discord(serenity::Error),
    Network(reqwest::Error),
    IO(io::Error),
    Encoding(str::Utf8Error),
}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Error {
        Error::Network(e)
    }
}

impl From<serenity::Error> for Error {
    fn from(e: serenity::Error) -> Error {
        match e {
            serenity::Error::Io(e) => Error::IO(e),
            e => Error::Discord(e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::IO(e)
    }
}

impl<W> From<io::IntoInnerError<W>> for Error {
    fn from(e: io::IntoInnerError<W>) -> Error {
        Error::IO(e.into())
    }
}

impl From<str::Utf8Error> for Error {
    fn from(e: str::Utf8Error) -> Error {
        Error::Encoding(e)
    }
}

type Result<T> = result::Result<T, Error>;

struct CommandContext {
    ctx: Context,
    is_global: bool,
    message: Message,
    guild_id: id::GuildId,
    channel_id: id::ChannelId,
}

impl CommandContext {
    fn say(&self, message: &str) -> Result<()> {
        for msg in message.chars().chunks(2000).into_iter() {
            let msg_str: String = msg.collect();
            self.channel_id.say(&self.ctx.http, &msg_str)?;
        }
        Ok(())
    }

    fn react(&self, reaction: ReactionType) -> Result<()> {
        self.message.react(&self.ctx.http, reaction)?;
        Ok(())
    }

    fn set_setting(&self, setting: Setting) -> Result<()> {
        let mut data = self.ctx.data.try_write().ok_or(Error::Lock)?;
        let global = data.get_mut::<GlobalSetting>().unwrap();
        let settings = global.get_mut_guild(&self.guild_id);

        if self.is_global {
            settings.set_all(setting);
        } else {
            settings.set_channel(&self.channel_id, setting);
        }

        Ok(())
    }

    fn get_global_setting(&self) -> Result<GlobalSetting> {
        let data = self.ctx.data.try_read().ok_or(Error::Lock)?;
        let settings = data.get::<GlobalSetting>().unwrap();
        Ok(settings.clone())
    }

    fn get_guild_setting(&self) -> Result<GuildSetting> {
        Ok(self
            .get_global_setting()?
            .get_mut_guild(&self.guild_id)
            .clone())
    }

    fn get_channel_setting(&self) -> Result<Setting> {
        Ok(self
            .get_guild_setting()?
            .get_channel(&self.channel_id)
            .clone())
    }

    fn determine_compiler_name(&self, spec: &str) -> Result<String> {
        if let Some((_, id)) = self
            .get_channel_setting()?
            .language_map
            .iter()
            .find(|(k, _)| k.as_str() == spec)
        {
            self.say(&format!("`{}`でコンパイルするよ！", id.compiler_name))?;
            Ok(id.compiler_name.to_string())
        } else {
            Ok(spec.to_string())
        }
    }

    fn get_default_options(&self, compiler_name: &str) -> Result<Option<Vec<String>>> {
        for (pred, opts) in self.get_channel_setting()?.default_compiler_options {
            if pred.match_with(compiler_name) {
                self.say(&format!("`{}`をオプションとして渡すよ！", opts.join(" ")))?;
                return Ok(Some(opts));
            }
        }
        Ok(None)
    }
}

struct RunbotHandler;

impl RunbotHandler {
    fn unhandled(&self, ctx: &CommandContext) -> Result<()> {
        ctx.say("何？")
    }

    fn say_compile_result(
        &self,
        ctx: &CommandContext,
        res: wandbox::compile::Response,
    ) -> Result<()> {
        let mut buf = strip_ansi_escapes::Writer::new(Vec::new());

        if let Some(msg) = res.compiler_message {
            writeln!(buf, "```{}```", msg)?;
        }

        if let Some(msg) = res.program_message {
            writeln!(buf, "```{}```", msg)?;
        }

        if let Some(s) = res.signal {
            writeln!(buf, "exited with signal: {}", s)?;
        }

        if let Some(s) = res.status {
            writeln!(buf, "exited with status code {}", s)?;
        }

        if let Some(s) = res.url {
            writeln!(buf, "{}", s)?;
        }

        let msg = buf.into_inner()?;
        ctx.say(str::from_utf8(&msg)?)
    }

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
!runbot remap           -- 言語名とコンパイラを紐付け
!runbot default-options -- デフォルトで使用するオプションを設定
!runbot list-languages  -- 言語を一覧
!runbot list            -- 言語に対応するコンパイラを一覧
!runbot run             -- 実行
!runbot run-save        -- 実行して保存
```
",
        )
    }

    fn command_show_setting(&self, ctx: &CommandContext) -> Result<()> {
        let settings = ctx.get_guild_setting()?;

        let msg = if ctx.is_global {
            settings.to_string()
        } else {
            settings.get_channel(&ctx.channel_id).to_string()
        };

        ctx.say(&msg)
    }

    fn command_auto(&self, ctx: &CommandContext, state: bool) -> Result<()> {
        let mut setting = Setting::default();
        setting.auto = Some(state);

        ctx.set_setting(setting)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_auto_save(&self, ctx: &CommandContext, state: bool) -> Result<()> {
        let mut setting = Setting::default();
        setting.auto_save = Some(state);

        ctx.set_setting(setting)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_remap(&self, ctx: &CommandContext, commandline: &[impl AsRef<str>]) -> Result<()> {
        let (lang, compiler) = match commandline {
            [lang, compiler] => (lang, compiler),
            _ => return self.unhandled(ctx),
        };

        let mut setting = Setting::default();
        setting.language_map =
            hashmap![lang.as_ref().to_owned() => compiler.as_ref().parse().into_ok()];

        ctx.set_setting(setting)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_default_options(
        &self,
        ctx: &CommandContext,
        commandline: &[impl AsRef<str>],
    ) -> Result<()> {
        let (first, rest) = match commandline.split_first() {
            Some(x) => x,
            None => return self.unhandled(ctx),
        };

        let mut setting = Setting::default();
        setting.default_compiler_options = hashmap![first.as_ref().parse().into_ok() => rest.into_iter().map(|o| o.as_ref().to_string()).collect()];

        ctx.set_setting(setting)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_list_languages(&self, ctx: &CommandContext) -> Result<()> {
        use wandbox::list::List;
        let List(cs) = wandbox::list()?;

        let msg = cs
            .into_iter()
            .map(|c| c.language)
            .collect::<HashSet<_>>()
            .into_iter()
            .join("\n");

        ctx.say(&msg)?;

        Ok(())
    }

    fn command_list(&self, ctx: &CommandContext, commandline: &[impl AsRef<str>]) -> Result<()> {
        use wandbox::list::List;
        let List(cs) = wandbox::list()?;

        let language = match commandline {
            [x] => x.as_ref(),
            _ => return self.unhandled(ctx),
        };

        let msg = cs
            .into_iter()
            .filter(|c| c.language == language)
            .map(|c| format!("{}: {} ({})", c.name, c.display_name, c.language))
            .join("\n");

        ctx.say(&msg)?;

        Ok(())
    }

    fn command_run(
        &self,
        ctx: &CommandContext,
        commandline: &[impl AsRef<str>],
        body: &str,
        save: bool,
    ) -> Result<()> {
        let block: CodeBlock = body.parse().into_ok();

        let (compiler_spec, options) = match commandline.split_first() {
            Some((spec, [])) => (Some(spec.as_ref().to_string()), None),
            Some((spec, opts)) => (
                Some(spec.as_ref().to_string()),
                Some(opts.into_iter().map(|o| o.as_ref().to_string()).collect()),
            ),
            None => (None, None),
        };

        let compiler = match compiler_spec.or(block.language) {
            Some(spec) => ctx.determine_compiler_name(&spec)?,
            None => return self.unhandled(ctx),
        };

        let options = match options.or(ctx.get_default_options(&compiler)?) {
            Some(opts) => opts.join("\n"),
            None => "".to_string(),
        };

        use wandbox::compile::Request;
        let req = Request {
            compiler,
            code: block.code,
            codes: Vec::new(),
            options: None,
            stdin: None,
            compiler_option_raw: Some(options),
            runtime_option_raw: None,
            save,
        };

        let res = wandbox::compile(&req)?;
        self.say_compile_result(ctx, res)
    }

    fn handle_implicit(&self, ctx: &CommandContext, content: &str) -> Result<()> {
        let setting = ctx.get_channel_setting()?;
        if !setting.auto() {
            return Ok(());
        }
        if !content.starts_with("```") || !content.ends_with("```") {
            return Ok(());
        }

        ctx.react(ReactionType::Unicode("👀".to_string()))?;

        let block: CodeBlock = content.parse().into_ok();

        let compiler = match block.language {
            Some(spec) => ctx.determine_compiler_name(&spec)?,
            None => return self.unhandled(ctx),
        };

        let options = match ctx.get_default_options(&compiler)? {
            Some(opts) => opts.join("\n"),
            None => "".to_string(),
        };

        let save = setting.auto_save();

        use wandbox::compile::Request;
        let req = Request {
            compiler,
            code: block.code,
            codes: Vec::new(),
            options: None,
            stdin: None,
            compiler_option_raw: Some(options),
            runtime_option_raw: None,
            save,
        };

        let res = wandbox::compile(&req)?;
        self.say_compile_result(ctx, res)
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
            Err(_) => return self.unhandled(ctx),
        };

        let (command, commandline) = match words.split_first() {
            Some(x) => x,
            None => return self.unhandled(ctx),
        };

        match command.as_ref() {
            "help" => self.command_help(ctx),
            "show-setting" => self.command_show_setting(ctx),
            "auto" => self.command_auto(ctx, true),
            "no-auto" => self.command_auto(ctx, false),
            "auto-save" => self.command_auto_save(ctx, true),
            "no-auto-save" => self.command_auto_save(ctx, false),
            "remap" => self.command_remap(ctx, commandline),
            "default-options" => self.command_default_options(ctx, commandline),
            "list-languages" => self.command_list_languages(ctx),
            "list" => self.command_list(ctx, commandline),
            "run" => self.command_run(ctx, commandline, body, false),
            "run-save" => self.command_run(ctx, commandline, body, true),
            _ => self.unhandled(ctx),
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
            Some(id) => id,
            None => return,
        };

        let mut command_ctx = CommandContext {
            ctx,
            is_global: false,
            message: msg.clone(),
            guild_id,
            channel_id: msg.channel_id.clone(),
        };

        if let Err(e) = self.handle(&mut command_ctx, &msg.content) {
            let _ = match e {
                Error::Lock => command_ctx.say("争いはよくないですよ"),
                Error::Network(_) => command_ctx.say("めのまえが まっくらに なった!"),
                Error::Discord(_) => command_ctx.say("ごめん"),
                Error::IO(_) => command_ctx.say("体調が悪いので休みます"),
                Error::Encoding(_) => command_ctx.say("ひーん"),
            };
            eprintln!("command returned an error: {:?}", e);
        }
    }

    fn ready(&self, _: Context, ready: Ready) {
        println!("Runbot ready! I'm {}", ready.user.name);
    }
}

fn main() {
    let token = env::var("DISCORD_TOKEN").unwrap();

    let mut client = Client::new(&token, RunbotHandler).unwrap();

    {
        let mut data = client.data.write();
        data.insert::<GlobalSetting>(GlobalSetting::new());
    }

    client.start().unwrap();
}
