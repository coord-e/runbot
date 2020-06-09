#![feature(str_strip)]
#![feature(never_type)]
#![feature(unwrap_infallible)]

use std::collections::{HashMap, HashSet};
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
struct GlobalSetting {
    inner: HashMap<id::ChannelId, Setting>,
    current_default: Setting,
}

impl fmt::Display for GlobalSetting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}

impl GlobalSetting {
    fn new() -> GlobalSetting {
        GlobalSetting {
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

impl TypeMapKey for GlobalSetting {
    type Value = GlobalSetting;
}

#[derive(Debug)]
enum Error {
    Lock,
    Discord(serenity::Error),
    Network(reqwest::Error),
    IO(io::Error),
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

type Result<T> = result::Result<T, Error>;

struct CommandContext {
    ctx: Context,
    is_global: bool,
    message: Message,
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
        let settings = data.get_mut::<GlobalSetting>().unwrap();
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

    fn get_channel_setting(&self) -> Result<Setting> {
        Ok(self
            .get_global_setting()?
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
        if let Some(msg) = res.compiler_message {
            ctx.say(&format!("```{}```", msg))?;
        }

        if let Some(msg) = res.program_message {
            ctx.say(&format!("```{}```", msg))?;
        }

        if let Some(s) = res.signal {
            ctx.say(&format!("exit with signal: {}", s))?;
        }

        if let Some(s) = res.status {
            ctx.say(&format!("exit with status code {}", s))?;
        }

        if let Some(s) = res.url {
            ctx.say(&s)?;
        }

        Ok(())
    }

    fn command_help(&self, ctx: &CommandContext) -> Result<()> {
        ctx.say("
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
")
    }

    fn command_show_setting(&self, ctx: &CommandContext) -> Result<()> {
        let settings = ctx.get_global_setting()?;

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

    fn command_remap(&self, ctx: &CommandContext, commandline: &[&str]) -> Result<()> {
        let (lang, compiler) = match commandline {
            &[lang, compiler] => (lang, compiler),
            _ => return self.unhandled(ctx),
        };

        let mut setting = Setting::default();
        setting.language_map = hashmap![lang.to_owned() => compiler.parse().into_ok()];

        ctx.set_setting(setting)?;

        ctx.react(ReactionType::Unicode("✅".to_string()))
    }

    fn command_default_options(&self, ctx: &CommandContext, commandline: &[&str]) -> Result<()> {
        let (first, rest) = match commandline.split_first() {
            Some(x) => x,
            None => return self.unhandled(ctx),
        };

        let mut setting = Setting::default();
        setting.default_compiler_options =
            hashmap![first.parse().into_ok() => rest.into_iter().map(|o| o.to_string()).collect()];

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

    fn command_list(&self, ctx: &CommandContext, commandline: &[&str]) -> Result<()> {
        use wandbox::list::List;
        let List(cs) = wandbox::list()?;

        if commandline.is_empty() {
            return self.unhandled(ctx);
        }
        let language = commandline.join(" ");

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
        commandline: &[&str],
        body: &str,
        save: bool,
    ) -> Result<()> {
        let block: CodeBlock = body.parse().into_ok();

        let (compiler_spec, options) = match commandline.split_first() {
            Some((spec, [])) => (Some(spec.to_string()), None),
            Some((spec, opts)) => (
                Some(spec.to_string()),
                Some(opts.into_iter().map(|o| o.to_string()).collect()),
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

    fn command_implicit(&self, ctx: &CommandContext, content: &str) -> Result<()> {
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
}

impl EventHandler for RunbotHandler {
    fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let mut command_ctx = CommandContext {
            ctx,
            is_global: false,
            message: msg.clone(),
            channel_id: msg.channel_id.clone(),
        };

        let mut lines = msg.content.lines();
        let line = lines.next().unwrap();
        let body = lines.join("\n");

        let result = if let Some(rest) = line.strip_prefix("!runbot") {
            command_ctx.react(ReactionType::Unicode("👀".to_string()));

            let rest = if let Some(rest) = rest.trim().strip_prefix("global") {
                command_ctx.is_global = true;
                rest
            } else {
                command_ctx.is_global = false;
                rest
            };

            let mut it = rest.trim().split(' ');
            if let Some(command) = it.next() {
                let commandline: Vec<_> = it.collect();
                match command {
                    "help" => self.command_help(&command_ctx),
                    "show-setting" => self.command_show_setting(&command_ctx),
                    "auto" => self.command_auto(&command_ctx, true),
                    "no-auto" => self.command_auto(&command_ctx, false),
                    "auto-save" => self.command_auto_save(&command_ctx, true),
                    "no-auto-save" => self.command_auto_save(&command_ctx, false),
                    "remap" => self.command_remap(&command_ctx, &commandline),
                    "default-options" => self.command_default_options(&command_ctx, &commandline),
                    "list-languages" => self.command_list_languages(&command_ctx),
                    "list" => self.command_list(&command_ctx, &commandline),
                    "run" => self.command_run(&command_ctx, &commandline, &body, false),
                    "run-save" => self.command_run(&command_ctx, &commandline, &body, true),
                    _ => self.unhandled(&command_ctx),
                }
            } else {
                self.unhandled(&command_ctx)
            }
        } else {
            self.command_implicit(&command_ctx, &msg.content)
        };

        if let Err(e) = result {
            match e {
                Error::Lock => command_ctx.say("争いはよくないですよ"),
                Error::Network(_) => command_ctx.say("めのまえが まっくらに なった!"),
                Error::Discord(_) => command_ctx.say("ごめん"),
                Error::IO(_) => command_ctx.say("体調が悪いので休みます"),
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
