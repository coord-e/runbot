use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crate::model::compiler::CompilerID;

use derive_more::{Constructor, Display};
use ref_cast::RefCast;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct LanguageID(u64);

impl LanguageID {
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn from_u64(id: u64) -> LanguageID {
        LanguageID(id)
    }
}

impl redis::ToRedisArgs for LanguageID {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: redis::RedisWrite,
    {
        self.as_u64().write_redis_args(out)
    }
}

impl redis::FromRedisValue for LanguageID {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        u64::from_redis_value(v).map(LanguageID)
    }
}

#[derive(Debug, Clone, RefCast, Display)]
#[repr(transparent)]
pub struct LanguageName(String);

impl FromStr for LanguageName {
    type Err = !;
    fn from_str(s: &str) -> Result<LanguageName, !> {
        Ok(LanguageName(s.to_owned()))
    }
}

impl PartialEq<LanguageName> for LanguageName {
    fn eq(&self, other: &LanguageName) -> bool {
        self.as_string().eq_ignore_ascii_case(other.as_string())
    }
}

impl Eq for LanguageName {}

impl Hash for LanguageName {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        for byte in self.as_string().bytes() {
            hasher.write_u8(byte.to_ascii_lowercase())
        }
    }
}

impl LanguageName {
    pub fn as_string(&self) -> &String {
        &self.0
    }

    pub fn from_string(s: String) -> LanguageName {
        LanguageName(s)
    }

    pub fn from_string_ref(s: &String) -> &LanguageName {
        Self::ref_cast(s)
    }
}

#[derive(Debug, Clone, Constructor)]
pub struct Language {
    id: LanguageID,
    name: LanguageName,
    aliases: HashSet<LanguageName>,
    default_compiler_id: Option<CompilerID>,
}

impl Language {
    pub fn id(&self) -> LanguageID {
        self.id
    }

    pub fn name(&self) -> &LanguageName {
        &self.name
    }

    pub fn aliases(&self) -> &HashSet<LanguageName> {
        &self.aliases
    }

    pub fn default_compiler_id(&self) -> Option<CompilerID> {
        self.default_compiler_id
    }

    pub fn is_named_as(&self, name: &LanguageName) -> bool {
        &self.name == name || self.aliases.contains(name)
    }
}
