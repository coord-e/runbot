#[derive(Debug, Clone, Copy)]
pub struct ChannelID(u64);

impl ChannelID {
    pub fn from_u64(id: u64) -> ChannelID {
        ChannelID(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}
