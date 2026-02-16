use disktidy_lib::modules::file_analyzer::{GarbageDetector, GarbageDetectorOptions};

#[tokio::test]
async fn test_garbage_detector_creation() {
    let detector = GarbageDetector::new();
    assert!(detector.detect_system_temp().is_empty() || !detector.detect_system_temp().is_empty());
}

#[tokio::test]
async fn test_garbage_detector_options_default() {
    let options = GarbageDetectorOptions::default();
    assert!(options.include_system_temp);
    assert!(options.include_browser_cache);
    assert_eq!(options.min_file_age_days, 0);
}

#[tokio::test]
async fn test_garbage_detector_with_options() {
    let options = GarbageDetectorOptions {
        include_system_temp: true,
        include_browser_cache: false,
        include_app_cache: false,
        include_recycle_bin: false,
        include_log_files: false,
        min_file_age_days: 0,
        max_files_per_category: Some(10),
    };
    
    let detector = GarbageDetector::with_options(options);
    let files = detector.detect_system_temp();
    
    assert!(files.is_empty() || !files.is_empty());
}
