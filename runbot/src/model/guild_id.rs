#[derive(Debug, Clone, Copy)]
pub struct GuildID(u64);

impl GuildID {
    pub fn from_u64(id: u64) -> GuildID {
        GuildID(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
