pub mod dump_setting;
pub mod list_compilers;
pub mod list_languages;
pub mod remap_language;
pub mod run;
pub mod run_implicit;
pub mod set_auto;
pub mod set_auto_save;

pub use dump_setting::dump_setting;
pub use list_compilers::list_compilers;
pub use list_languages::list_languages;
pub use remap_language::remap_language;
pub use run::run;
pub use run_implicit::run_implicit;
pub use set_auto::set_auto;
pub use set_auto_save::set_auto_save;

use crate::config::Config;
use crate::model::channel_id::ChannelID;
use crate::model::compiler::{Compiler, CompilerName};
use crate::model::compiler_spec::CompilerSpec;
use crate::model::guild_id::GuildID;
use crate::model::language::{Language, LanguageName};
use crate::setting::Setting;
use crate::{Error, Result};

pub struct ActionContext {
    setting: Setting,
    config: Config,
    guild_id: GuildID,
    channel_id: ChannelID,
}

impl ActionContext {
    pub fn new(
        setting: Setting,
        config: Config,
        guild_id: GuildID,
        channel_id: ChannelID,
    ) -> ActionContext {
        ActionContext {
            setting,
            config,
            guild_id,
            channel_id,
        }
    }

    pub fn channel_id(&self) -> ChannelID {
        self.channel_id
    }

    pub fn resolve_compiler_spec(&self, spec: &CompilerSpec) -> Result<&Compiler> {
        if let Some(language) = self.config.find_language(spec.as_language_name()) {
            self.resolve_language(language)
        } else if let Some(compiler) = self.config.find_compiler(spec.as_compiler_name()) {
            Ok(compiler)
        } else {
            Err(Error::UnknownCompilerSpec(spec.clone()))
        }
    }

    pub fn resolve_language_name(&self, language_name: &LanguageName) -> Result<&Compiler> {
        if let Some(language) = self.config.find_language(language_name) {
            self.resolve_language(language)
        } else {
            Err(Error::UnknownLanguageName(language_name.clone()))
        }
    }

    fn resolve_language(&self, language: &Language) -> Result<&Compiler> {
        if let Some(compiler_id) = self
            .setting
            .get_remap(self.guild_id, self.channel_id, language.id())?
            .or_else(|| language.default_compiler_id())
        {
            Ok(self.config.get_compiler(compiler_id))
        } else {
            Err(Error::UnmappedLanguage(language.name().clone()))
        }
    }

    pub fn is_auto(&self) -> Result<bool> {
        self.setting.get_auto(self.guild_id, self.channel_id)
    }

    pub fn is_auto_save(&self) -> Result<bool> {
        self.setting.get_auto_save(self.guild_id, self.channel_id)
    }

    pub fn all_remap(&self) -> Result<Vec<(&LanguageName, &CompilerName)>> {
        let remaps = self
            .setting
            .get_remap_all(self.guild_id, self.channel_id)?
            .into_iter()
            .map(|(language_id, compiler_id)| {
                (
                    self.config.get_language(language_id).name(),
                    self.config.get_compiler(compiler_id).name(),
                )
            })
            .collect();
        Ok(remaps)
    }
}
