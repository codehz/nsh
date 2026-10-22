//! The per-engine store behind `blob:` URLs: `URL.createObjectURL` puts a
//! blob's bytes here, and everything that accepts a URL reads them back from
//! it — `fetch`, images (`src`, `placeholder`, `Image.getSize`), canvas
//! `drawImage` and `Clipboard.writeImage` — until the URL is revoked.
//!
//! The store is shared by the whole engine: the JS host writes into it, the
//! core reads it (`ui::ambient::blob_bytes`, the clipboard) and the platform
//! backend the engine mounts into reads it when it loads an image. Bytes are
//! never copied into a `data:` URI, so a 48 MB clipboard image costs one copy
//! in the store and none across the mount batch.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

/// The bytes and type of a `blob:` URL.
#[derive(Clone)]
pub struct BlobEntry {
    /// The URL that names it (`blob:nsh://app/…`).
    pub url: String,
    /// Its bytes.
    pub bytes: Arc<[u8]>,
    /// Its MIME type (`""` when unknown).
    pub mime: String,
}

// The URL is the identity; comparing image bytes on every `Location`
// comparison would be needless.
impl PartialEq for BlobEntry {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Eq for BlobEntry {}

impl std::fmt::Debug for BlobEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobEntry")
            .field("url", &self.url)
            .field("mime", &self.mime)
            .field("bytes", &self.bytes.len())
            .finish()
    }
}

/// One engine's blobs, by `blob:` URL. Clones share the same store.
#[derive(Clone, Default)]
pub struct BlobStore {
    entries: Arc<Mutex<HashMap<String, BlobEntry>>>,
}

impl std::fmt::Debug for BlobStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobStore")
            .field("entries", &self.entries())
            .finish()
    }
}

impl BlobStore {
    /// Stores `bytes` as `mime` and returns the URL that names them.
    pub fn put(&self, bytes: impl Into<Arc<[u8]>>, mime: impl Into<String>) -> String {
        let url = new_url();
        let entry = BlobEntry {
            url: url.clone(),
            bytes: bytes.into(),
            mime: mime.into(),
        };
        self.lock().insert(url.clone(), entry);
        url
    }

    /// The entry `url` names, if it is live. A fragment is ignored.
    pub fn get(&self, url: &str) -> Option<BlobEntry> {
        let url = strip_fragment(url);
        self.lock().get(url).cloned()
    }

    /// Removes `url`; true when it named a live entry. A fragment is ignored.
    pub fn remove(&self, url: &str) -> bool {
        let url = strip_fragment(url);
        self.lock().remove(url).is_some()
    }

    /// Whether `url` names a live entry (`get(url).is_some()`).
    pub fn has(&self, url: &str) -> bool {
        self.get(url).is_some()
    }

    /// How many entries are live.
    fn entries(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, BlobEntry>> {
        // A panic while holding it leaves the map consistent: nothing between
        // the borrow and the insert / remove may unwind.
        self.entries.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// `url` without its fragment: a `blob:` URL names the same bytes with one.
fn strip_fragment(url: &str) -> &str {
    url.split('#').next().unwrap_or(url)
}

/// A URL of the form `blob:nsh://app/<32 hex digits>`, fresh on every call.
fn new_url() -> String {
    use std::{
        hash::{BuildHasher, Hasher},
        sync::atomic::{AtomicU64, Ordering},
    };
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let mut h = std::hash::RandomState::new().build_hasher();
    h.write_u64(COUNTER.fetch_add(1, Ordering::Relaxed));
    let a = h.finish();
    h.write_u64(a);
    format!("blob:nsh://app/{a:016x}{:016x}", h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_remove() {
        let store = BlobStore::default();
        let url = store.put(vec![1, 2, 3], "image/png");
        assert!(url.starts_with("blob:nsh://app/"), "{url}");
        assert!(store.has(&url));
        let entry = store.get(&url).expect("live");
        assert_eq!(
            (entry.url.as_str(), entry.mime.as_str()),
            (url.as_str(), "image/png")
        );
        assert_eq!(&entry.bytes[..], &[1, 2, 3]);
        assert!(store.remove(&url));
        assert!(store.get(&url).is_none());
        assert!(!store.remove(&url));
    }

    #[test]
    fn fragments_name_the_same_entry() {
        let store = BlobStore::default();
        let url = store.put(vec![7], "text/plain");
        let with_fragment = format!("{url}#frag");
        assert_eq!(store.get(&with_fragment).expect("live").url, url);
        assert!(store.remove(&with_fragment));
        assert!(!store.has(&url));
    }

    #[test]
    fn urls_are_distinct() {
        let store = BlobStore::default();
        let a = store.put(Vec::new(), "");
        let b = store.put(Vec::new(), "");
        assert_ne!(a, b);
        assert!(store.has(&a) && store.has(&b));
    }

    #[test]
    fn empty_bytes_are_valid() {
        let store = BlobStore::default();
        let url = store.put(Vec::new(), "");
        let entry = store.get(&url).expect("live");
        assert!(entry.bytes.is_empty());
        assert!(entry.mime.is_empty());
    }

    #[test]
    fn clones_share_entries() {
        let store = BlobStore::default();
        let other = store.clone();
        let url = other.put(vec![9], "image/png");
        assert!(store.has(&url));
        assert!(store.remove(&url));
        assert!(!other.has(&url));
    }

    #[test]
    fn unknown_urls_are_absent() {
        let store = BlobStore::default();
        assert!(store.get("blob:nsh://app/0").is_none());
        assert!(store.get("https://example.com/a.png").is_none());
    }
}
