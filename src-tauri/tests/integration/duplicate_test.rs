use disktidy_lib::modules::file_analyzer::DuplicateDetector;
use disktidy_lib::models::DuplicateDetectorOptions;

#[tokio::test]
async fn test_duplicate_detector_creation() {
    let detector = DuplicateDetector::new();
    assert!(detector.find_duplicates(&[]).total_files == 0);
}

#[tokio::test]
async fn test_duplicate_detector_options_default() {
    let options = DuplicateDetectorOptions::default();
    assert!(options.min_size > 0);
    assert!(!options.include_hidden);
    assert!(options.use_cache);
}

#[tokio::test]
async fn test_duplicate_detector_with_options() {
    let options = DuplicateDetectorOptions {
        min_size: u64::MAX,
        max_size: None,
        include_hidden: false,
        use_cache: true,
    };
    
    let detector = DuplicateDetector::with_options(options);
    let result = detector.find_duplicates(&[]);
    
    assert_eq!(result.total_files, 0);
}

#[tokio::test]
async fn test_duplicate_suggestions_empty() {
    let detector = DuplicateDetector::new();
    let empty_group = disktidy_lib::models::DuplicateGroup {
        hash: String::new(),
        size: 0,
        files: vec![],
        wasted_space: 0,
        file_type: String::new(),
    };
    let suggestions = detector.suggest_files_to_delete(&empty_group);
    assert!(suggestions.is_empty());
}
