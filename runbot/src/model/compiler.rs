use std::str::FromStr;

use crate::model::language::LanguageID;

use derive_more::{Constructor, Display};
use ref_cast::RefCast;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct CompilerID(u64);

impl CompilerID {
    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn from_u64(id: u64) -> CompilerID {
        CompilerID(id)
    }
}

impl redis::ToRedisArgs for CompilerID {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: redis::RedisWrite,
    {
        self.as_u64().write_redis_args(out)
    }
}

impl redis::FromRedisValue for CompilerID {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        u64::from_redis_value(v).map(CompilerID)
    }
}

#[derive(Debug, Clone, RefCast, PartialEq, Eq, Display)]
#[repr(transparent)]
pub struct CompilerName(String);

impl FromStr for CompilerName {
    type Err = !;
    fn from_str(s: &str) -> Result<CompilerName, !> {
        Ok(CompilerName(s.to_owned()))
    }
}

impl CompilerName {
    pub fn as_string(&self) -> &String {
        &self.0
    }

    pub fn from_string(s: String) -> CompilerName {
        CompilerName(s)
    }

    #[allow(clippy::ptr_arg)]
    pub fn from_string_ref(s: &String) -> &CompilerName {
        Self::ref_cast(s)
    }
}

#[derive(Debug, Clone, Display)]
pub struct CompilerVersion(String);

impl CompilerVersion {
    pub fn as_string(&self) -> &String {
        &self.0
    }

    pub fn from_string(s: String) -> CompilerVersion {
        CompilerVersion(s)
    }
}

#[derive(Debug, Clone, Constructor)]
pub struct Compiler {
    id: CompilerID,
    name: CompilerName,
    version: Option<CompilerVersion>,
    language_id: LanguageID,
    wandbox_name: String,
}

impl Compiler {
    pub fn id(&self) -> CompilerID {
        self.id
    }

    pub fn name(&self) -> &CompilerName {
        &self.name
    }

    pub fn version(&self) -> Option<&CompilerVersion> {
        self.version.as_ref()
    }

    pub fn language_id(&self) -> LanguageID {
        self.language_id
    }

    pub fn wandbox_name(&self) -> &String {
        &self.wandbox_name
    }
}
