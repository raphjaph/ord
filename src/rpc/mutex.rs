
use std::sync::{Mutex, MutexGuard};

lazy_static::lazy_static! {
    pub static ref INSCRIBE_MUTEX: Mutex<usize> = Mutex::new(0);
}

pub fn lock_inscribe() -> MutexGuard<'static, usize> {
    INSCRIBE_MUTEX.lock().unwrap()
}