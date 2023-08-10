use std::time::{self, UNIX_EPOCH};

pub fn now() -> usize {
    time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_secs() as usize
}
