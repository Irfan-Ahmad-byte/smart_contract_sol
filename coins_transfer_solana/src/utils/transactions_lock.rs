use std::sync::Mutex;

extern crate lazy_static;

lazy_static::lazy_static! {
    static ref TRANSACTIONS_LOCKS_ARRAY: Mutex<Vec<i64>> = Mutex::new(Vec::new());
}

pub struct TransactionsLocks {}

impl TransactionsLocks {
    pub fn is_locked(value: i64) -> bool {
        let array = TRANSACTIONS_LOCKS_ARRAY.lock().unwrap();
        array.contains(&value)
    }

    pub fn add_lock(value: i64) {
        let mut array = TRANSACTIONS_LOCKS_ARRAY.lock().unwrap();
        array.push(value);
    }
    pub fn remove_lock(value: i64) {
        let mut array = TRANSACTIONS_LOCKS_ARRAY.lock().unwrap();
        if let Some(pos) = array.iter().position(|&x| x == value) {
            array.remove(pos);
        }
    }
}
