use std::sync::Arc;

use crate::model::channel_id::ChannelID;
use crate::model::compiler::CompilerID;
use crate::model::guild_id::GuildID;
use crate::model::language::LanguageID;
use crate::Result;

use parking_lot::Mutex;
use redis::Commands;

// Data Access Object for Runbot settings
pub struct Setting {
    conn: Arc<Mutex<redis::Connection>>,
}

#[derive(Debug, Clone, Copy)]
pub enum Scope {
    Channel(ChannelID),
    Guild,
}

#[derive(Debug, Clone, Copy)]
enum Field {
    Auto,
    AutoSave,
    Remap,
}

impl Field {
    fn name(&self) -> &'static str {
        match self {
            Field::Auto => "auto",
            Field::AutoSave => "auto_save",
            Field::Remap => "remap",
        }
    }
}

#[derive(Debug, Clone)]
enum Key {
    ChannelKey {
        guild_id: GuildID,
        channel_id: ChannelID,
        field: Field,
    },
    DefaultKey {
        guild_id: GuildID,
        field: Field,
    },
}

impl redis::ToRedisArgs for Key {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: redis::RedisWrite,
    {
        let key_name = match self {
            Key::ChannelKey {
                guild_id,
                channel_id,
                field,
            } => format!(
                "channel:{}:{}:{}",
                guild_id.as_u64(),
                channel_id.as_u64(),
                field.name()
            ),
            Key::DefaultKey { guild_id, field } => {
                format!("channel:{}:default:{}", guild_id.as_u64(), field.name())
            }
        };

        key_name.write_redis_args(out)
    }
}

enum ScanPattern {
    AllChannels(GuildID, Field),
}

impl redis::ToRedisArgs for ScanPattern {
    fn write_redis_args<W: ?Sized>(&self, out: &mut W)
    where
        W: redis::RedisWrite,
    {
        match self {
            ScanPattern::AllChannels(guild_id, field) => {
                format!("channel:{}:*:{}", guild_id.as_u64(), field.name()).write_redis_args(out)
            }
        }
    }
}

impl Setting {
    pub fn new(conn: Arc<Mutex<redis::Connection>>) -> Setting {
        Setting { conn }
    }

    fn get_key(
        &self,
        guild_id: GuildID,
        channel_id: ChannelID,
        field: Field,
    ) -> Result<Option<Key>> {
        let channel_key = Key::ChannelKey {
            guild_id,
            channel_id,
            field,
        };
        let default_key = Key::DefaultKey { guild_id, field };

        let key = if self.conn.lock().exists(channel_key.clone())? {
            Some(channel_key)
        } else if self.conn.lock().exists(default_key.clone())? {
            Some(default_key)
        } else {
            None
        };

        Ok(key)
    }

    fn get_simple<V: redis::FromRedisValue>(
        &self,
        guild_id: GuildID,
        channel_id: ChannelID,
        field: Field,
    ) -> Result<Option<V>> {
        let key = match self.get_key(guild_id, channel_id, field)? {
            Some(k) => k,
            None => return Ok(None),
        };
        let value = self.conn.lock().get(key)?;
        Ok(Some(value))
    }

    fn get_hash<K: redis::ToRedisArgs, V: redis::FromRedisValue>(
        &self,
        guild_id: GuildID,
        channel_id: ChannelID,
        field: Field,
        hash_key: K,
    ) -> Result<Option<V>> {
        let key = match self.get_key(guild_id, channel_id, field)? {
            Some(k) => k,
            None => return Ok(None),
        };
        let value = self.conn.lock().hget(key, hash_key)?;
        Ok(value)
    }

    fn get_hash_all<K: redis::FromRedisValue, V: redis::FromRedisValue>(
        &self,
        guild_id: GuildID,
        channel_id: ChannelID,
        field: Field,
    ) -> Result<Vec<(K, V)>> {
        let key = match self.get_key(guild_id, channel_id, field)? {
            Some(k) => k,
            None => return Ok(Vec::new()),
        };
        let value = self.conn.lock().hgetall(key)?;
        Ok(value)
    }

    fn set_simple<V>(&self, guild_id: GuildID, scope: Scope, field: Field, value: V) -> Result<()>
    where
        V: redis::ToRedisArgs + Copy,
    {
        match scope {
            Scope::Channel(channel_id) => {
                let key = Key::ChannelKey {
                    guild_id,
                    channel_id,
                    field,
                };
                self.conn.lock().set(key, value)?;
            }
            Scope::Guild => {
                let mut conn = self.conn.lock();

                let keys: Vec<Vec<u8>> = conn
                    .scan_match::<_, Vec<u8>>(ScanPattern::AllChannels(guild_id, field))?
                    .collect();

                for k in keys {
                    conn.set(k, value)?;
                }

                let default_key = Key::DefaultKey { guild_id, field };
                conn.set(default_key, value)?;
            }
        }

        Ok(())
    }

    fn set_hash<K, V>(
        &self,
        guild_id: GuildID,
        scope: Scope,
        field: Field,
        hash_key: K,
        hash_value: V,
    ) -> Result<()>
    where
        K: redis::ToRedisArgs + Copy,
        V: redis::ToRedisArgs + Copy,
    {
        match scope {
            Scope::Channel(channel_id) => {
                let key = Key::ChannelKey {
                    guild_id,
                    channel_id,
                    field,
                };
                self.conn.lock().hset(key, hash_key, hash_value)?;
            }
            Scope::Guild => {
                let mut conn = self.conn.lock();

                let keys: Vec<Vec<u8>> = conn
                    .scan_match::<_, Vec<u8>>(ScanPattern::AllChannels(guild_id, field))?
                    .collect();

                for k in keys {
                    conn.hset(k, hash_key, hash_value)?;
                }

                let default_key = Key::DefaultKey { guild_id, field };
                conn.hset(default_key, hash_key, hash_value)?;
            }
        }

        Ok(())
    }

    // public interface
    pub fn set_auto(&self, guild_id: GuildID, scope: Scope, set: bool) -> Result<()> {
        self.set_simple(guild_id, scope, Field::Auto, set as u32)
    }

    pub fn get_auto(&self, guild_id: GuildID, channel_id: ChannelID) -> Result<bool> {
        let data: u32 = self
            .get_simple(guild_id, channel_id, Field::Auto)?
            .unwrap_or(1);
        Ok(data != 0)
    }

    pub fn set_auto_save(&self, guild_id: GuildID, scope: Scope, set: bool) -> Result<()> {
        self.set_simple(guild_id, scope, Field::AutoSave, set as u32)
    }

    pub fn get_auto_save(&self, guild_id: GuildID, channel_id: ChannelID) -> Result<bool> {
        let data: u32 = self
            .get_simple(guild_id, channel_id, Field::AutoSave)?
            .unwrap_or(0);
        Ok(data != 0)
    }

    pub fn set_remap(
        &self,
        guild_id: GuildID,
        scope: Scope,
        language_id: LanguageID,
        compiler_id: CompilerID,
    ) -> Result<()> {
        self.set_hash(guild_id, scope, Field::Remap, language_id, compiler_id)
    }

    pub fn get_remap(
        &self,
        guild_id: GuildID,
        channel_id: ChannelID,
        language_id: LanguageID,
    ) -> Result<Option<CompilerID>> {
        self.get_hash(guild_id, channel_id, Field::Remap, language_id)
    }

    pub fn get_remap_all(
        &self,
        guild_id: GuildID,
        channel_id: ChannelID,
    ) -> Result<Vec<(LanguageID, CompilerID)>> {
        self.get_hash_all(guild_id, channel_id, Field::Remap)
    }
}
