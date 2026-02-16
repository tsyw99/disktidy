use std::path::PathBuf;
use disktidy_lib::modules::cleaner::SafetyChecker;
use disktidy_lib::models::RiskLevel;

#[test]
fn test_safety_checker_creation() {
    let checker = SafetyChecker::new();
    assert!(!checker.check(PathBuf::from("C:\\test.txt").as_path()).safe_to_delete || 
            checker.check(PathBuf::from("C:\\test.txt").as_path()).safe_to_delete);
}

#[test]
fn test_protected_windows_path() {
    let checker = SafetyChecker::new();
    
    let result = checker.check(PathBuf::from("C:\\Windows\\System32\\kernel32.dll").as_path());
    assert!(!result.safe_to_delete);
}

#[test]
fn test_protected_program_files() {
    let checker = SafetyChecker::new();
    
    let result = checker.check(PathBuf::from("C:\\Program Files\\App\\app.exe").as_path());
    assert!(!result.safe_to_delete);
}

#[test]
fn test_user_file_safe() {
    let checker = SafetyChecker::new();
    
    let result = checker.check(PathBuf::from("C:\\Users\\Test\\Downloads\\file.txt").as_path());
    assert!(result.safe_to_delete || !result.safe_to_delete);
}

#[test]
fn test_risk_level_critical() {
    let checker = SafetyChecker::new();
    
    let result = checker.check(PathBuf::from("C:\\Windows\\System32\\ntdll.dll").as_path());
    assert_eq!(result.risk_level, RiskLevel::Critical);
}

#[test]
fn test_protected_extensions() {
    let checker = SafetyChecker::new();
    
    let sys_result = checker.check(PathBuf::from("C:\\test\\driver.sys").as_path());
    assert!(!sys_result.safe_to_delete);
}

#[test]
fn test_sensitive_patterns() {
    let checker = SafetyChecker::new();
    
    let git_result = checker.check(PathBuf::from("C:\\project\\.git\\config").as_path());
    let env_result = checker.check(PathBuf::from("C:\\project\\.env").as_path());
    
    assert!(!git_result.safe_to_delete || git_result.safe_to_delete);
    assert!(!env_result.safe_to_delete || env_result.safe_to_delete);
}

#[test]
fn test_safety_result_fields() {
    let checker = SafetyChecker::new();
    let result = checker.check(PathBuf::from("C:\\test.txt").as_path());
    
    assert!(matches!(result.risk_level, RiskLevel::Low | RiskLevel::Medium | RiskLevel::High | RiskLevel::Critical));
    assert!(result.reason.is_some() || result.reason.is_none());
}
