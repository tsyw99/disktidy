#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

pub struct TestFixture {
    pub temp_dir: TempDir,
    pub test_files: Vec<PathBuf>,
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            temp_dir: TempDir::new().unwrap(),
            test_files: Vec::new(),
        }
    }

    pub fn create_file(&mut self, name: &str, content: &[u8]) -> PathBuf {
        let path = self.temp_dir.path().join(name);
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        
        let mut file = File::create(&path).unwrap();
        file.write_all(content).unwrap();
        
        self.test_files.push(path.clone());
        path
    }

    pub fn create_file_with_size(&mut self, name: &str, size: usize) -> PathBuf {
        let content = vec![0u8; size];
        self.create_file(name, &content)
    }

    pub fn create_temp_file(&mut self, extension: &str) -> PathBuf {
        let name = format!("test_{}.{}", uuid::Uuid::new_v4(), extension);
        self.create_file(&name, b"test content")
    }

    pub fn create_duplicate_files(&mut self, count: usize, size: usize) -> Vec<PathBuf> {
        let content = vec![0xABu8; size];
        let mut paths = Vec::new();

        for i in 0..count {
            let name = format!("duplicate_{}.bin", i);
            paths.push(self.create_file(&name, &content));
        }

        paths
    }

    pub fn create_garbage_files(&mut self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        paths.push(self.create_file("temp.tmp", b"temp content"));
        paths.push(self.create_file("cache.tmp", b"cache content"));
        paths.push(self.create_file("log.log", b"log content"));
        paths.push(self.create_file("backup.bak", b"backup content"));

        paths
    }

    pub fn create_large_files(&mut self, count: usize, size_mb: usize) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let size = size_mb * 1024 * 1024;

        for i in 0..count {
            let name = format!("large_{}.bin", i);
            paths.push(self.create_file_with_size(&name, size));
        }

        paths
    }

    pub fn temp_path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn file_count(&self) -> usize {
        self.test_files.len()
    }

    pub fn clear(&mut self) {
        self.test_files.clear();
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

pub fn assert_file_not_exists(path: &Path) {
    assert!(!path.exists(), "File should not exist: {:?}", path);
}

pub fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "File should exist: {:?}", path);
}

pub fn assert_file_size(path: &Path, expected_size: u64) {
    let metadata = fs::metadata(path).unwrap();
    assert_eq!(metadata.len(), expected_size, "File size mismatch");
}
