use std::path::PathBuf;
use std::sync::Arc;
use disktidy_lib::modules::cleaner::CleanerExecutor;
use disktidy_lib::models::{CleanOptions, CleanProgress};

#[tokio::test]
async fn test_cleaner_executor_creation() {
    let executor = CleanerExecutor::new();
    assert!(executor.clean(vec![]).await.is_ok());
}

#[tokio::test]
async fn test_cleaner_executor_with_options() {
    let options = CleanOptions {
        move_to_recycle_bin: true,
        secure_delete: false,
        secure_pass_count: 3,
    };
    
    let executor = CleanerExecutor::with_options(options);
    let result = executor.clean(vec![]).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clean_with_progress() {
    let executor = CleanerExecutor::new();
    let progress_called = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let progress_flag = progress_called.clone();
    
    let callback = move |_progress: CleanProgress| {
        progress_flag.store(true, std::sync::atomic::Ordering::SeqCst);
    };
    
    let _result = executor.clean_with_progress(vec![], Some(Arc::new(callback))).await;
}

#[tokio::test]
async fn test_clean_nonexistent_file() {
    let executor = CleanerExecutor::new();
    let result = executor.clean(vec![PathBuf::from("C:\\nonexistent_file_12345.tmp")]).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clean_options_default() {
    let options = CleanOptions::default();
    
    assert!(options.move_to_recycle_bin);
    assert!(!options.secure_delete);
    assert_eq!(options.secure_pass_count, 3);
}

#[tokio::test]
async fn test_clean_empty_list() {
    let executor = CleanerExecutor::new();
    let result = executor.clean(vec![]).await.unwrap();
    
    assert_eq!(result.cleaned_files, 0);
    assert_eq!(result.total_files, 0);
}
