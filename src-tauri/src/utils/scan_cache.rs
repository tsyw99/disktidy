use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SCAN_CACHE_FILE: &str = "app_cache_scan.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCache {
    pub last_scan_time: u64,
    pub app_caches: HashMap<String, AppScanInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppScanInfo {
    pub last_scan: u64,
    pub scanned_paths: Vec<String>,
    pub file_count: u64,
    pub total_size: u64,
}

impl Default for ScanCache {
    fn default() -> Self {
        Self {
            last_scan_time: 0,
            app_caches: HashMap::new(),
        }
    }
}

impl ScanCache {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn load(cache_dir: &Path) -> Self {
        let cache_file = cache_dir.join(SCAN_CACHE_FILE);
        if cache_file.exists() {
            if let Ok(content) = fs::read_to_string(&cache_file) {
                if let Ok(cache) = serde_json::from_str(&content) {
                    return cache;
                }
            }
        }
        Self::default()
    }
    
    pub fn save(&self, cache_dir: &Path) -> Result<(), String> {
        if !cache_dir.exists() {
            fs::create_dir_all(cache_dir)
                .map_err(|e| format!("Failed to create cache directory: {}", e))?;
        }
        
        let cache_file = cache_dir.join(SCAN_CACHE_FILE);
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize cache: {}", e))?;
        
        fs::write(&cache_file, content)
            .map_err(|e| format!("Failed to write cache file: {}", e))?;
        
        Ok(())
    }
    
    pub fn update_app_info(&mut self, app: &str, paths: Vec<String>, file_count: u64, total_size: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        
        self.app_caches.insert(app.to_string(), AppScanInfo {
            last_scan: now,
            scanned_paths: paths,
            file_count,
            total_size,
        });
        
        self.last_scan_time = now;
    }
    
    pub fn get_app_last_scan(&self, app: &str) -> Option<u64> {
        self.app_caches.get(app).map(|info| info.last_scan)
    }
    
    pub fn should_rescan(&self, app: &str, force: bool) -> bool {
        if force {
            return true;
        }
        
        if let Some(info) = self.app_caches.get(app) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            
            let cache_validity_seconds: u64 = 3600;
            now - info.last_scan > cache_validity_seconds
        } else {
            true
        }
    }
}

#[derive(Debug, Clone)]
pub struct IncrementalScanState {
    pub last_scan_timestamp: u64,
    pub scanned_files: u64,
    pub modified_files: u64,
}

impl IncrementalScanState {
    pub fn new(last_scan: u64) -> Self {
        Self {
            last_scan_timestamp: last_scan,
            scanned_files: 0,
            modified_files: 0,
        }
    }
    
    pub fn should_scan_file(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }
        
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    return duration.as_secs() >= self.last_scan_timestamp;
                }
            }
        }
        
        true
    }
    
    pub fn record_file(&mut self, _path: &Path, is_modified: bool) {
        self.scanned_files += 1;
        if is_modified {
            self.modified_files += 1;
        }
    }
}

pub fn get_cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|p| p.join("DiskTidy"))
}

pub fn get_current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
