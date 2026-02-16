use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, Instant};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::time::timeout;
use lru::LruCache;
use std::sync::Mutex;

use crate::models::DiskTidyError;

const BUFFER_SIZE: usize = 64 * 1024;
const PARTIAL_HASH_SIZE: usize = 64 * 1024;
const DEFAULT_CACHE_SIZE: usize = 1000;
const DEFAULT_TIMEOUT_SECS: u64 = 30;
const LARGE_FILE_THRESHOLD: u64 = 100 * 1024 * 1024;

#[derive(Clone)]
struct CachedHash {
    hash: String,
    modified_time: SystemTime,
    file_size: u64,
}

#[derive(Debug, Clone)]
pub struct HashResult {
    pub hash: String,
    pub is_partial: bool,
    pub bytes_processed: u64,
}

pub struct HashCalculator {
    cache: Mutex<LruCache<PathBuf, CachedHash>>,
    cache_enabled: bool,
    timeout_secs: u64,
}

impl HashCalculator {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap())),
            cache_enabled: true,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    pub fn with_cache(enabled: bool) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap())),
            cache_enabled: enabled,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    pub fn with_cache_and_size(enabled: bool, max_size: usize) -> Self {
        let size = NonZeroUsize::new(max_size.max(1)).unwrap_or(NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap());
        Self {
            cache: Mutex::new(LruCache::new(size)),
            cache_enabled: enabled,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(NonZeroUsize::new(DEFAULT_CACHE_SIZE).unwrap())),
            cache_enabled: true,
            timeout_secs,
        }
    }

    pub async fn calculate_file_hash(&self, path: &Path) -> Result<HashResult, DiskTidyError> {
        let path_string = path.to_string_lossy().to_string();
        
        let result = timeout(
            Duration::from_secs(self.timeout_secs),
            self.calculate_file_hash_internal(path)
        ).await;

        match result {
            Ok(Ok(hash_result)) => Ok(hash_result),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(DiskTidyError::HashCalculationTimeout { path: path_string }),
        }
    }

    async fn calculate_file_hash_internal(&self, path: &Path) -> Result<HashResult, DiskTidyError> {
        let mut file = File::open(path)
            .await
            .map_err(DiskTidyError::IoError)?;

        let metadata = file.metadata().await.map_err(DiskTidyError::IoError)?;
        let modified_time = metadata.modified().map_err(DiskTidyError::IoError)?;
        let file_size = metadata.len();

        if self.cache_enabled {
            let mut cache = self.cache.lock().unwrap();
            if let Some(cached) = cache.get(path) {
                if cached.modified_time == modified_time && cached.file_size == file_size {
                    return Ok(HashResult {
                        hash: cached.hash.clone(),
                        is_partial: false,
                        bytes_processed: file_size,
                    });
                }
            }
        }

        if file_size > LARGE_FILE_THRESHOLD {
            return self.calculate_large_file_hash(path, file, file_size, modified_time).await;
        }

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let mut total_bytes = 0u64;

        loop {
            let bytes_read = file
                .read(&mut buffer)
                .await
                .map_err(DiskTidyError::IoError)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
            total_bytes += bytes_read as u64;
        }

        let hash = format!("{:x}", hasher.finalize());

        if self.cache_enabled {
            let mut cache = self.cache.lock().unwrap();
            cache.put(path.to_path_buf(), CachedHash {
                hash: hash.clone(),
                modified_time,
                file_size,
            });
        }

        Ok(HashResult {
            hash,
            is_partial: false,
            bytes_processed: total_bytes,
        })
    }

    async fn calculate_large_file_hash(
        &self,
        path: &Path,
        mut file: File,
        file_size: u64,
        modified_time: SystemTime,
    ) -> Result<HashResult, DiskTidyError> {
        let start_time = Instant::now();
        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; BUFFER_SIZE];
        let mut total_bytes = 0u64;
        let timeout_duration = Duration::from_secs(self.timeout_secs);

        loop {
            if start_time.elapsed() > timeout_duration {
                return Err(DiskTidyError::HashCalculationTimeout {
                    path: path.to_string_lossy().to_string(),
                });
            }

            let bytes_read = file
                .read(&mut buffer)
                .await
                .map_err(DiskTidyError::IoError)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
            total_bytes += bytes_read as u64;
        }

        let hash = format!("{:x}", hasher.finalize());

        if self.cache_enabled {
            let mut cache = self.cache.lock().unwrap();
            cache.put(path.to_path_buf(), CachedHash {
                hash: hash.clone(),
                modified_time,
                file_size,
            });
        }

        Ok(HashResult {
            hash,
            is_partial: false,
            bytes_processed: total_bytes,
        })
    }

    pub async fn calculate_partial_hash(&self, path: &Path) -> Result<HashResult, DiskTidyError> {
        let mut file = File::open(path)
            .await
            .map_err(DiskTidyError::IoError)?;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; PARTIAL_HASH_SIZE];
        let bytes_read = file
            .read(&mut buffer)
            .await
            .map_err(DiskTidyError::IoError)?;

        hasher.update(&buffer[..bytes_read]);

        Ok(HashResult {
            hash: format!("{:x}", hasher.finalize()),
            is_partial: true,
            bytes_processed: bytes_read as u64,
        })
    }

    pub async fn quick_compare(&self, path1: &Path, path2: &Path) -> Result<bool, DiskTidyError> {
        let hash1 = self.calculate_partial_hash(path1).await?;
        let hash2 = self.calculate_partial_hash(path2).await?;

        if hash1.hash != hash2.hash {
            return Ok(false);
        }

        let full_hash1 = self.calculate_file_hash(path1).await?;
        let full_hash2 = self.calculate_file_hash(path2).await?;

        Ok(full_hash1.hash == full_hash2.hash)
    }

    pub fn clear_cache(&mut self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    pub fn cache_size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }
}

impl Default for HashCalculator {
    fn default() -> Self {
        Self::new()
    }
}

pub fn hash_string(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn hash_bytes(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}
