use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use once_cell::sync::Lazy;
use crate::modules::cancellable_file_classifier::{
    FileClassificationResult, FileClassificationOptions, CancellableFileClassifier,
};

static CANCEL_FLAG: Lazy<Arc<AtomicBool>> = 
    Lazy::new(|| Arc::new(AtomicBool::new(false)));

#[tauri::command]
pub async fn classify_files(
    path: String,
    options: Option<FileClassificationOptions>,
) -> FileClassificationResult {
    let opts = options.unwrap_or_default();
    let classifier = CancellableFileClassifier::with_options(opts);
    classifier.classify(&path)
}

#[tauri::command]
pub async fn classify_disk(disk: String) -> FileClassificationResult {
    let path = if disk == "all" {
        "C:\\".to_string()
    } else {
        format!("{}\\", disk.trim_end_matches('\\'))
    };

    let options = FileClassificationOptions {
        max_depth: Some(5),
        include_hidden: false,
        include_system: false,
        exclude_paths: vec![
            "Windows".to_string(),
            "Program Files".to_string(),
            "Program Files (x86)".to_string(),
            "ProgramData".to_string(),
            "$Recycle.Bin".to_string(),
            "System Volume Information".to_string(),
        ],
        max_files: Some(100000),
        top_n_categories: Some(15),
    };

    let classifier = CancellableFileClassifier::with_options(options);
    classifier.classify(&path)
}

#[tauri::command]
pub async fn start_classify_files(
    path: String,
    options: Option<FileClassificationOptions>,
) -> FileClassificationResult {
    // Reset cancel flag before starting
    CANCEL_FLAG.store(false, Ordering::SeqCst);
    
    let opts = options.unwrap_or_default();
    let classifier = CancellableFileClassifier::with_options_and_flag(opts, Arc::clone(&CANCEL_FLAG));
    
    classifier.classify(&path)
}

#[tauri::command]
pub async fn cancel_classify_files() -> bool {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
    true
}
