use crate::{Error, Result};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Serialize, de::DeserializeOwned};
use std::path::Path;

const ENTRIES: TableDefinition<'_, &str, &[u8]> = TableDefinition::new("entries");
const DIGESTS: TableDefinition<'_, &str, &str> = TableDefinition::new("digests");

/// A content hash of a source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Digest(String);

impl Digest {
    /// Hash arbitrary bytes.
    #[must_use]
    pub fn of(bytes: &[u8]) -> Self {
        Self(blake3::hash(bytes).to_hex().to_string())
    }

    /// The digest as hex.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Whether a write actually changed anything.
///
/// `No` is the mechanism that keeps a full refetch from rebuilding a whole site: an unchanged
/// digest dirties nothing downstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Changed {
    /// The entry was written.
    Yes,
    /// The digest matched; nothing was written.
    No,
}

/// A digest-keyed content store.
#[derive(Debug)]
pub struct Store {
    db: Database,
}

fn store_err(e: impl std::fmt::Display) -> Error {
    Error::Store(e.to_string())
}

impl Store {
    /// Open, or create, a store at `path`.
    ///
    /// # Errors
    ///
    /// Fails if the file cannot be created or is not a store.
    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::create(path).map_err(store_err)?;
        let txn = db.begin_write().map_err(store_err)?;
        {
            txn.open_table(ENTRIES).map_err(store_err)?;
            txn.open_table(DIGESTS).map_err(store_err)?;
        }
        txn.commit().map_err(store_err)?;
        Ok(Self { db })
    }

    /// The digest recorded for `key`, if any.
    ///
    /// # Errors
    ///
    /// Fails if the store cannot be read.
    pub fn digest_of(&self, key: &str) -> Result<Option<Digest>> {
        let txn = self.db.begin_read().map_err(store_err)?;
        let table = txn.open_table(DIGESTS).map_err(store_err)?;
        let found = table.get(key).map_err(store_err)?;
        Ok(found.map(|v| Digest(v.value().to_owned())))
    }

    /// Every recorded digest, read in a single transaction.
    ///
    /// A per-key read costs a transaction setup each time, which is what made a full content
    /// rescan scale linearly with an expensive constant. One transaction makes the scan a hash
    /// lookup per file.
    ///
    /// # Errors
    ///
    /// Fails if the store cannot be read.
    pub fn all_digests(&self) -> Result<std::collections::HashMap<String, Digest>> {
        let txn = self.db.begin_read().map_err(store_err)?;
        let table = txn.open_table(DIGESTS).map_err(store_err)?;
        let mut out = std::collections::HashMap::new();
        for row in table.iter().map_err(store_err)? {
            let (key, value) = row.map_err(store_err)?;
            out.insert(key.value().to_owned(), Digest(value.value().to_owned()));
        }
        Ok(out)
    }

    /// Write `value` under `key` unless its digest already matches.
    ///
    /// # Errors
    ///
    /// Fails if the value cannot be encoded or the store cannot be written.
    pub fn set<T: Serialize>(&self, key: &str, value: &T, digest: &Digest) -> Result<Changed> {
        if self.digest_of(key)?.as_ref() == Some(digest) {
            return Ok(Changed::No);
        }
        let encoded = serde_json::to_vec(value)?;
        let txn = self.db.begin_write().map_err(store_err)?;
        {
            let mut entries = txn.open_table(ENTRIES).map_err(store_err)?;
            entries.insert(key, encoded.as_slice()).map_err(store_err)?;
            let mut digests = txn.open_table(DIGESTS).map_err(store_err)?;
            digests.insert(key, digest.as_str()).map_err(store_err)?;
        }
        txn.commit().map_err(store_err)?;
        Ok(Changed::Yes)
    }

    /// Read the entry stored under `key`.
    ///
    /// # Errors
    ///
    /// Fails if the store cannot be read or the value cannot be decoded.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let txn = self.db.begin_read().map_err(store_err)?;
        let table = txn.open_table(ENTRIES).map_err(store_err)?;
        let Some(found) = table.get(key).map_err(store_err)? else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_slice(found.value())?))
    }

    /// Every key in the store, in sorted order.
    ///
    /// Sorted because build output must not depend on iteration order
    /// (`docs/concepts/determinism.md`, rule 2).
    ///
    /// # Errors
    ///
    /// Fails if the store cannot be read.
    pub fn keys(&self) -> Result<Vec<String>> {
        let txn = self.db.begin_read().map_err(store_err)?;
        let table = txn.open_table(ENTRIES).map_err(store_err)?;
        let mut keys = Vec::new();
        for row in table.iter().map_err(store_err)? {
            let (key, _) = row.map_err(store_err)?;
            keys.push(key.value().to_owned());
        }
        keys.sort();
        Ok(keys)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn temp_store(name: &str) -> (Store, std::path::PathBuf) {
        let path = std::env::temp_dir().join(format!("tri-store-{name}.redb"));
        let _ = std::fs::remove_file(&path);
        (Store::open(&path).expect("opens"), path)
    }

    #[test]
    fn unchanged_digests_write_nothing() {
        let (store, path) = temp_store("unchanged");
        let digest = Digest::of(b"hello");
        assert_eq!(
            store.set("blog/a", &"first", &digest).expect("set"),
            Changed::Yes
        );
        assert_eq!(
            store.set("blog/a", &"first", &digest).expect("set"),
            Changed::No
        );

        let moved = Digest::of(b"hello, again");
        assert_eq!(
            store.set("blog/a", &"second", &moved).expect("set"),
            Changed::Yes
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn round_trips_values_and_sorts_keys() {
        let (store, path) = temp_store("roundtrip");
        for id in ["blog/c", "blog/a", "blog/b"] {
            store
                .set(id, &id.to_owned(), &Digest::of(id.as_bytes()))
                .expect("set");
        }
        assert_eq!(
            store.get::<String>("blog/b").expect("get"),
            Some("blog/b".to_owned())
        );
        assert_eq!(
            store.keys().expect("keys"),
            vec!["blog/a", "blog/b", "blog/c"]
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn digests_differ_for_different_bytes() {
        assert_ne!(Digest::of(b"a"), Digest::of(b"b"));
        assert_eq!(Digest::of(b"a"), Digest::of(b"a"));
    }
}
