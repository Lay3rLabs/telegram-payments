#![allow(dead_code)]
use thiserror::Error;

use crate::wasi::keyvalue::{atomics, store};

const LOCKS_BUCKET: &str = "locks";
const LOCK_KEY: &str = "global_lock";

const OFFSET_BUCKET: &str = "offsets";
const OFFSET_KEY: &str = "latest_offset";

pub fn acquire_lock() -> KvStoreResult<atomics::Cas> {
    // Create a CAS handle for the lock key
    let cas = open_cas(LOCKS_BUCKET, LOCK_KEY)?;

    // Check if lock is currently held
    match cas.current() {
        Ok(Some(value)) => {
            // Check the actual lock value
            if value == b"locked" {
                // Lock is already held by someone else
                Err(KvStoreError::KeyNotFound(
                    "Lock already acquired".to_string(),
                ))
            } else {
                // Lock exists but is empty/unlocked, try to acquire it
                match atomics::swap(cas, b"locked") {
                    Ok(()) => {
                        // Successfully acquired the lock, create new CAS handle to return
                        open_cas(LOCKS_BUCKET, LOCK_KEY)
                    }
                    Err(e) => {
                        // Failed to acquire lock (race condition or other error)
                        Err(KvStoreError::AtomicSwap {
                            bucket: LOCKS_BUCKET.to_string(),
                            key: LOCK_KEY.to_string(),
                            reason: e.to_string(),
                        })
                    }
                }
            }
        }
        Ok(None) | Err(_) => {
            // Lock doesn't exist yet, try to acquire it
            match atomics::swap(cas, b"locked") {
                Ok(()) => {
                    // Successfully acquired the lock, create new CAS handle to return
                    open_cas(LOCKS_BUCKET, LOCK_KEY)
                }
                Err(e) => {
                    // Failed to acquire lock (race condition or other error)
                    Err(KvStoreError::AtomicSwap {
                        bucket: LOCKS_BUCKET.to_string(),
                        key: LOCK_KEY.to_string(),
                        reason: e.to_string(),
                    })
                }
            }
        }
    }
}

pub fn release_lock(cas: atomics::Cas) -> KvStoreResult<()> {
    // Release the lock by swapping to empty value
    atomics::swap(cas, b"").map_err(|e| KvStoreError::AtomicSwap {
        bucket: LOCKS_BUCKET.to_string(),
        key: LOCK_KEY.to_string(),
        reason: e.to_string(),
    })
}

pub fn get_offset() -> KvStoreResult<Option<i64>> {
    let value = match read_value(OFFSET_BUCKET, OFFSET_KEY) {
        Err(KvStoreError::MissingKey { .. }) => {
            return Ok(None);
        }
        Err(e) => {
            return Err(e);
        }
        Ok(value) => value,
    };

    let bytes: [u8; 8] = value.try_into().map_err(|_| KvStoreError::ReadKey {
        bucket: OFFSET_BUCKET.to_string(),
        key: OFFSET_KEY.to_string(),
        reason: "Invalid data format: expected 8 bytes for i64".to_string(),
    })?;
    let n = i64::from_le_bytes(bytes);

    Ok(Some(n))
}

pub fn set_offset(offset: i64) -> KvStoreResult<()> {
    let value = offset.to_le_bytes();
    write_value(OFFSET_BUCKET, OFFSET_KEY, &value)
}

fn read_value(bucket_id: &str, key: &str) -> KvStoreResult<Vec<u8>> {
    let bucket = open_bucket(bucket_id)?;
    bucket
        .get(key)
        .map_err(|e| KvStoreError::ReadKey {
            bucket: bucket_id.to_string(),
            key: key.to_string(),
            reason: e.to_string(),
        })?
        .ok_or_else(|| KvStoreError::MissingKey {
            bucket: bucket_id.to_string(),
            key: key.to_string(),
        })
}

fn write_value(bucket_id: &str, key: &str, value: &[u8]) -> KvStoreResult<()> {
    let bucket = open_bucket(bucket_id)?;
    bucket.set(key, value).map_err(|e| KvStoreError::WriteKey {
        bucket: bucket_id.to_string(),
        key: key.to_string(),
        reason: e.to_string(),
    })
}

fn atomic_swap(bucket_id: &str, key: &str, value: &[u8]) -> KvStoreResult<()> {
    let cas = open_cas(bucket_id, key)?;
    atomics::swap(cas, value).map_err(|e| KvStoreError::AtomicSwap {
        bucket: bucket_id.to_string(),
        key: key.to_string(),
        reason: e.to_string(),
    })
}

fn atomic_read(bucket_id: &str, key: &str) -> KvStoreResult<Vec<u8>> {
    let cas = open_cas(bucket_id, key)?;
    cas.current()
        .map_err(|e| KvStoreError::AtomicRead {
            bucket: bucket_id.to_string(),
            key: key.to_string(),
            reason: e.to_string(),
        })?
        .ok_or_else(|| KvStoreError::MissingKey {
            bucket: bucket_id.to_string(),
            key: key.to_string(),
        })
}

fn open_cas(id: &str, key: &str) -> KvStoreResult<atomics::Cas> {
    let bucket = open_bucket(id)?;
    atomics::Cas::new(&bucket, key).map_err(|e| KvStoreError::AtomicCasResource {
        bucket: id.to_string(),
        key: key.to_string(),
        reason: e.to_string(),
    })
}

fn open_bucket(id: &str) -> KvStoreResult<store::Bucket> {
    store::open(id).map_err(|e| KvStoreError::BucketOpen {
        id: id.to_string(),
        reason: e.to_string(),
    })
}

#[derive(Error, Debug)]
pub enum KvStoreError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("IoError: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to open bucket {id}: {reason}")]
    BucketOpen { id: String, reason: String },
    #[error("Failed to read key {key} for bucket {bucket}: {reason}")]
    ReadKey {
        bucket: String,
        key: String,
        reason: String,
    },
    #[error("Failed to writekey {key} for bucket {bucket}: {reason}")]
    WriteKey {
        bucket: String,
        key: String,
        reason: String,
    },
    #[error("Missing key: {key} for bucket {bucket}")]
    MissingKey { bucket: String, key: String },
    #[error("Failed to atomically increment bucket {bucket}, key {key}, delta {delta}: {reason}")]
    AtomicIncrement {
        bucket: String,
        key: String,
        delta: i64,
        reason: String,
    },
    #[error("Failed to atomically swap bucket {bucket}, key {key}: {reason}")]
    AtomicSwap {
        bucket: String,
        key: String,
        reason: String,
    },
    #[error("Failed to acquire atomic CAS lock for bucket {bucket}, key {key}: {reason}")]
    AtomicCasResource {
        bucket: String,
        key: String,
        reason: String,
    },
    #[error("Failed to read atomic value for bucket {bucket}, key {key}: {reason}")]
    AtomicRead {
        bucket: String,
        key: String,
        reason: String,
    },
    #[error("Failed to perform batch operation for bucket {bucket}, {reason}")]
    BatchRead { bucket: String, reason: String },
    #[error("Failed to perform batch write for bucket {bucket}, {reason}")]
    BatchWrite { bucket: String, reason: String },
    #[error("Failed to perform batch delete for bucket {bucket}, {reason}")]
    BatchDelete { bucket: String, reason: String },
    #[error("Failed to list keys for bucket {bucket}, cursor: {cursor:?}: {reason}")]
    ListKeys {
        bucket: String,
        cursor: Option<String>,
        reason: String,
    },
}

pub type KvStoreResult<T> = Result<T, KvStoreError>;
