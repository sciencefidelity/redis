use bytes::Bytes;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(crate) struct DbDropGuard {
    db: Db,
}

#[derive(Clone, Debug)]
pub(crate) struct Db {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
}

#[derive(Debug)]
struct State {
    entries: HashMap<String, Entry>,
}

#[derive(Debug)]
struct Entry {
    data: Bytes,
}

impl DbDropGuard {
    pub(crate) fn new() -> Self {
        DbDropGuard { db: Db::new() }
    }

    pub(crate) fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Db {
    pub(crate) fn new() -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                entries: HashMap::new(),
            }),
        });
        Db { shared }
    }

    pub(crate) fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared.state.lock().unwrap();
        state.entries.get(key).map(|entry| entry.data.clone())
    }

    pub(crate) fn set(&self, key: String, value: Bytes) {
        let mut state = self.shared.state.lock().unwrap();

        let _prev = state.entries.insert(key.clone(), Entry { data: value });

        drop(state);
    }
}
