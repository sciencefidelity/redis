use bytes::Bytes;
use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;
use tokio::time::{self, Duration, Instant};

#[derive(Debug)]
pub struct DropGuard {
    db: Db,
}

#[derive(Clone, Debug)]
pub struct Db {
    shared: Arc<Shared>,
}

#[derive(Debug)]
struct Shared {
    state: Mutex<State>,
    background_task: Notify,
}

#[derive(Debug)]
struct State {
    entries: HashMap<String, Entry>,
    expirations: BTreeSet<(Instant, String)>,
    shutdown: bool,
}

#[derive(Debug)]
struct Entry {
    data: Bytes,
    expires_at: Option<Instant>,
}

impl DropGuard {
    pub(crate) fn new() -> Self {
        Self { db: Db::new() }
    }

    pub(crate) fn db(&self) -> Db {
        self.db.clone()
    }
}

impl Drop for DropGuard {
    fn drop(&mut self) {
        self.db.shutdown_purge_task();
    }
}

impl Db {
    pub(crate) fn new() -> Self {
        let shared = Arc::new(Shared {
            state: Mutex::new(State {
                entries: HashMap::new(),
                expirations: BTreeSet::new(),
                shutdown: false,
            }),
            background_task: Notify::new(),
        });

        tokio::spawn(purge_expired_tasks(Arc::clone(&shared)));

        Self { shared }
    }

    pub(crate) fn get(&self, key: &str) -> Option<Bytes> {
        let state = self.shared.state.lock().expect("failed to read db state");
        state.entries.get(key).map(|entry| entry.data.clone())
    }

    pub(crate) fn set(&self, key: String, value: Bytes, expire: Option<Duration>) {
        let mut state = self.shared.state.lock().expect("failed to read db state");

        let mut notify = false;

        let expires_at = expire.map(|duration| {
            let when = Instant::now() + duration;

            notify = state
                .next_expiration()
                .map_or(true, |expiration| expiration > when);

            when
        });

        let prev = state.entries.insert(
            key.clone(),
            Entry {
                data: value,
                expires_at,
            },
        );

        if let Some(prev) = prev {
            if let Some(when) = prev.expires_at {
                state.expirations.remove(&(when, key.clone()));
            }
        }

        if let Some(when) = expires_at {
            state.expirations.insert((when, key));
        }

        drop(state);

        if notify {
            self.shared.background_task.notify_one();
        }
    }

    fn shutdown_purge_task(&self) {
        let mut state = self.shared.state.lock().expect("failed to read db state");
        state.shutdown = true;

        drop(state);
        self.shared.background_task.notify_one();
    }
}

impl Shared {
    #[allow(clippy::significant_drop_tightening)]
    fn purge_expired_keys(&self) -> Option<Instant> {
        let mut state = self.state.lock().expect("failed to read state");

        let state = &mut *state;

        let now = Instant::now();

        while let Some(&(when, ref key)) = state.expirations.iter().next() {
            if when > now {
                return Some(when);
            }

            state.entries.remove(key);
            state.expirations.remove(&(when, key.clone()));
        }

        None
    }

    fn is_shutdown(&self) -> bool {
        self.state.lock().expect("failed to read state").shutdown
    }
}

impl State {
    fn next_expiration(&self) -> Option<Instant> {
        self.expirations
            .iter()
            .next()
            .map(|expiration| expiration.0)
    }
}

async fn purge_expired_tasks(shared: Arc<Shared>) {
    while !shared.is_shutdown() {
        if let Some(when) = shared.purge_expired_keys() {
            tokio::select! {
                () = time::sleep_until(when) => {},
                () = shared.background_task.notified() => {}
            }
        } else {
            shared.background_task.notified().await;
        }
    }
}
