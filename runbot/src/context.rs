use std::sync::Arc;

use crate::model::channel_id::ChannelID;
use crate::model::compiler::{Compiler, CompilerName};
use crate::model::compiler_spec::CompilerSpec;
use crate::model::guild_id::GuildID;
use crate::model::language::{Language, LanguageName};
use crate::setting::Setting;
use crate::table::Table;
use crate::{Error, Result};

use parking_lot::Mutex;

pub struct Context {
    pub(crate) setting: Setting,
    pub(crate) table: Table,
    pub(crate) wandbox: wandbox::blocking::Client,
    pub(crate) guild_id: GuildID,
    pub(crate) channel_id: ChannelID,
}

impl Context {
    pub fn new(
        guild_id: GuildID,
        channel_id: ChannelID,
        wandbox_client: wandbox::blocking::Client,
        redis_connection: Arc<Mutex<redis::Connection>>,
        redis_prefix: String,
        table: Table,
    ) -> Context {
        let setting = Setting::new(redis_connection, redis_prefix);
        Context {
            setting,
            table,
            wandbox: wandbox_client,
            guild_id,
            channel_id,
        }
    }

    pub(crate) fn resolve_compiler_spec(&self, spec: &CompilerSpec) -> Result<&Compiler> {
        if let Some(language) = self.table.find_language(spec.as_language_name()) {
            self.resolve_language(language)
        } else if let Some(compiler) = self.table.find_compiler(spec.as_compiler_name()) {
            Ok(compiler)
        } else {
            Err(Error::UnknownCompilerSpec(spec.clone()))
        }
    }

    pub(crate) fn resolve_language_name(&self, language_name: &LanguageName) -> Result<&Compiler> {
        if let Some(language) = self.table.find_language(language_name) {
            self.resolve_language(language)
        } else {
            Err(Error::UnknownLanguageName(language_name.clone()))
        }
    }

    pub(crate) fn resolve_language(&self, language: &Language) -> Result<&Compiler> {
        if let Some(compiler_id) = self
            .setting
            .get_remap(self.guild_id, self.channel_id, language.id())?
            .or_else(|| language.default_compiler_id())
        {
            Ok(self.table.get_compiler(compiler_id))
        } else {
            Err(Error::UnmappedLanguage(language.name().clone()))
        }
    }

    pub(crate) fn is_auto(&self) -> Result<bool> {
        self.setting.get_auto(self.guild_id, self.channel_id)
    }

    pub(crate) fn is_auto_save(&self) -> Result<bool> {
        self.setting.get_auto_save(self.guild_id, self.channel_id)
    }

    pub(crate) fn all_remap(&self) -> Result<Vec<(&LanguageName, &CompilerName)>> {
        let remaps = self
            .setting
            .get_remap_all(self.guild_id, self.channel_id)?
            .into_iter()
            .map(|(language_id, compiler_id)| {
                (
                    self.table.get_language(language_id).name(),
                    self.table.get_compiler(compiler_id).name(),
                )
            })
            .collect();
        Ok(remaps)
    }
}
