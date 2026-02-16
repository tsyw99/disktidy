use std::path::PathBuf;
use disktidy_lib::modules::file_analyzer::LargeFileAnalyzer;
use disktidy_lib::models::LargeFileAnalyzerOptions;

#[tokio::test]
async fn test_large_file_analyzer_creation() {
    let analyzer = LargeFileAnalyzer::new();
    assert!(analyzer.analyze(&[]).total_files == 0);
}

#[tokio::test]
async fn test_large_file_analyzer_options_default() {
    let options = LargeFileAnalyzerOptions::default();
    assert!(options.threshold > 0);
    assert!(!options.include_hidden);
    assert!(!options.include_system);
}

#[tokio::test]
async fn test_large_file_analyzer_with_options() {
    let options = LargeFileAnalyzerOptions {
        threshold: u64::MAX,
        exclude_paths: vec![],
        include_hidden: false,
        include_system: false,
    };
    
    let analyzer = LargeFileAnalyzer::with_options(options);
    let result = analyzer.analyze(&[]);
    
    assert_eq!(result.total_files, 0);
}

#[tokio::test]
async fn test_large_file_stats_empty() {
    let analyzer = LargeFileAnalyzer::new();
    let stats = analyzer.calculate_stats(&[]);
    
    assert_eq!(stats.total_count, 0);
    assert_eq!(stats.total_size, 0);
    assert!(stats.largest_file.is_none());
}

#[tokio::test]
async fn test_large_file_details_nonexistent() {
    let analyzer = LargeFileAnalyzer::new();
    let details = analyzer.get_file_details(PathBuf::from("C:\\nonexistent_file_12345.txt").as_path());
    assert!(details.is_none());
}
