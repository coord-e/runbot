use std::sync::Arc;

use parking_lot::Mutex;
use serenity::prelude::TypeMapKey;

pub struct ConnectionKey;

impl TypeMapKey for ConnectionKey {
    type Value = Arc<Mutex<redis::Connection>>;
}
