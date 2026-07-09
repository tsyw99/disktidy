use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use super::scanner::ScannedFile;

/// 分类规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_folder: String,
    pub extensions: Vec<String>,
    pub min_size: Option<u64>,
    pub max_size: Option<u64>,
    pub date_range_days: Option<u64>,
    pub keywords: Vec<String>,
    pub enabled: bool,
}

impl Default for CategoryRule {
    fn default() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: String::new(),
            description: String::new(),
            target_folder: String::new(),
            extensions: vec![],
            min_size: None,
            max_size: None,
            date_range_days: None,
            keywords: vec![],
            enabled: true,
        }
    }
}

/// 按内容特征的分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentGroup {
    pub group_id: String,
    pub group_name: String,
    pub files: Vec<ScannedFile>,
    pub total_size: u64,
    pub similarity_score: f64,
    pub representative_keywords: Vec<String>,
}

/// 最终分类结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    /// 按规则分类的结果
    pub rule_groups: HashMap<String, Vec<ScannedFile>>,
    /// 按内容相似度自动归类的结果
    pub content_groups: Vec<ContentGroup>,
    /// 未能分类的文件
    pub unclassified: Vec<ScannedFile>,
    /// 规则统计
    pub rule_stats: Vec<CategoryRuleStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRuleStats {
    pub rule_id: String,
    pub rule_name: String,
    pub target_folder: String,
    pub file_count: usize,
    pub total_size: u64,
}

/// 内容分类器 - 按内容特征分类
pub struct ContentClassifier {
    rules: Vec<CategoryRule>,
    similarity_threshold: f64,
}

impl ContentClassifier {
    pub fn new(rules: Vec<CategoryRule>) -> Self {
        Self {
            rules,
            similarity_threshold: 0.5,
        }
    }

    pub fn with_similarity_threshold(mut self, threshold: f64) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// 分类文件
    pub fn classify(&self, files: &[ScannedFile]) -> ClassificationResult {
        let mut rule_groups: HashMap<String, Vec<ScannedFile>> = HashMap::new();
        let mut unclassified = Vec::new();
        let mut classified_paths = HashSet::new();

        // 1. 按规则分类
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let mut matched = Vec::new();
            for file in files {
                if classified_paths.contains(&file.path) {
                    continue;
                }
                if self.match_rule(file, rule) {
                    matched.push(file.clone());
                    classified_paths.insert(file.path.clone());
                }
            }

            if !matched.is_empty() {
                let key = rule.target_folder.clone();
                rule_groups.entry(key)
                    .and_modify(|v| v.extend(matched.clone()))
                    .or_insert(matched);
            }
        }

        // 2. 按扩展名分组（内容相似度归类）
        let ext_groups = self.group_by_extension(files, &classified_paths);
        let content_groups = self.build_content_groups(&ext_groups);

        // 3. 未分类的文件
        for file in files {
            if !classified_paths.contains(&file.path) {
                unclassified.push(file.clone());
            }
        }

        // 4. 生成规则统计
        let rule_stats: Vec<CategoryRuleStats> = self.rules.iter().map(|r| {
            let count = rule_groups.get(&r.target_folder)
                .map(|v| v.len())
                .unwrap_or(0);
            let total_size = rule_groups.get(&r.target_folder)
                .map(|v| v.iter().map(|f| f.size_bytes).sum())
                .unwrap_or(0);

            CategoryRuleStats {
                rule_id: r.id.clone(),
                rule_name: r.name.clone(),
                target_folder: r.target_folder.clone(),
                file_count: count,
                total_size,
            }
        }).collect();

        ClassificationResult {
            rule_groups,
            content_groups,
            unclassified,
            rule_stats,
        }
    }

    /// 额外分类：按内容相似度将未分类文件分组
    pub fn classify_unclassified(
        &self,
        files: &[ScannedFile],
        similarity_threshold: f64,
    ) -> Vec<ContentGroup> {
        let ext_groups = self.group_by_extension(files, &HashSet::new());
        self.build_content_groups_with_threshold(&ext_groups, similarity_threshold)
    }

    fn match_rule(&self, file: &ScannedFile, rule: &CategoryRule) -> bool {
        // 扩展名匹配
        if !rule.extensions.is_empty() {
            if !rule.extensions.contains(&file.extension.to_lowercase()) {
                return false;
            }
        }

        // 大小匹配
        if let Some(min) = rule.min_size {
            if file.size_bytes < min {
                return false;
            }
        }
        if let Some(max) = rule.max_size {
            if file.size_bytes > max {
                return false;
            }
        }

        // 日期范围匹配
        if let Some(days) = rule.date_range_days {
            let now = std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let cutoff = now.saturating_sub(days * 86400);
            if file.modified_time < cutoff {
                return false;
            }
        }

        // 关键词匹配（文件名中）
        if !rule.keywords.is_empty() {
            let name_lower = file.name.to_lowercase();
            if !rule.keywords.iter().any(|kw| name_lower.contains(&kw.to_lowercase())) {
                return false;
            }
        }

        true
    }

    fn group_by_extension(
        &self,
        files: &[ScannedFile],
        exclude: &HashSet<String>,
    ) -> HashMap<String, Vec<ScannedFile>> {
        let mut groups: HashMap<String, Vec<ScannedFile>> = HashMap::new();
        for file in files {
            if exclude.contains(&file.path) {
                continue;
            }
            let ext = file.extension.clone();
            groups.entry(ext).or_default().push(file.clone());
        }
        groups
    }

    fn build_content_groups(
        &self,
        ext_groups: &HashMap<String, Vec<ScannedFile>>,
    ) -> Vec<ContentGroup> {
        self.build_content_groups_with_threshold(ext_groups, self.similarity_threshold)
    }

    fn build_content_groups_with_threshold(
        &self,
        ext_groups: &HashMap<String, Vec<ScannedFile>>,
        threshold: f64,
    ) -> Vec<ContentGroup> {
        let mut groups = Vec::new();

        for (ext, files) in ext_groups {
            if files.len() < 2 {
                continue;
            }

            // 同类扩展名的文件作为一组
            let total_size: u64 = files.iter().map(|f| f.size_bytes).sum();
            let group_name = if ext.is_empty() {
                "无扩展名文件".to_string()
            } else {
                format!("{} 文件组", ext.to_uppercase())
            };

            // 计算简单的内容相似度（基于文件名和大小）
            let avg_size = total_size as f64 / files.len() as f64;
            let mut size_variance = 0.0;
            for f in files {
                let diff = f.size_bytes as f64 - avg_size;
                size_variance += diff * diff;
            }
            size_variance /= files.len() as f64;

            // 大小越均匀，相似度越高
            let similarity = if avg_size > 0.0 {
                (1.0 - (size_variance.sqrt() / avg_size).min(1.0)).max(0.0)
            } else {
                1.0
            };

            // 文件名相似度
            let name_similarity = self.calculate_name_similarity(files);

            let combined_score = similarity * 0.4 + name_similarity * 0.6;

            if combined_score >= threshold {
                let keywords = self.extract_group_keywords(files);
                groups.push(ContentGroup {
                    group_id: uuid::Uuid::new_v4().to_string(),
                    group_name,
                    files: files.clone(),
                    total_size,
                    similarity_score: combined_score,
                    representative_keywords: keywords,
                });
            }
        }

        // 按相似度降序
        groups.sort_by(|a, b| b.similarity_score.partial_cmp(&a.similarity_score).unwrap_or(std::cmp::Ordering::Equal));
        groups
    }

    fn calculate_name_similarity(&self, files: &[ScannedFile]) -> f64 {
        if files.len() < 2 {
            return 1.0;
        }

        // 检查是否有共同文件名前缀
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        let mut prefix_match_count = 0;
        let comparisons = files.len() * (files.len() - 1) / 2;

        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                let common_prefix = common_prefix_len(names[i], names[j]);
                let min_len = names[i].len().min(names[j].len()).max(1);
                if common_prefix as f64 / min_len as f64 > 0.5 {
                    prefix_match_count += 1;
                }
            }
        }

        if comparisons > 0 {
            prefix_match_count as f64 / comparisons as f64
        } else {
            1.0
        }
    }

    fn extract_group_keywords(&self, files: &[ScannedFile]) -> Vec<String> {
        // 统计文件名中的常见词
        let mut word_freq: HashMap<String, usize> = HashMap::new();
        for file in files {
            let name = file.name.to_lowercase();
            // 简单分词
            for part in name.split(|c: char| !c.is_alphanumeric()) {
                if part.len() >= 2 {
                    *word_freq.entry(part.to_string()).or_insert(0) += 1;
                }
            }
        }

        let mut words: Vec<(String, usize)> = word_freq.into_iter().collect();
        words.sort_by(|a, b| b.1.cmp(&a.1));
        words.truncate(5);
        words.into_iter().map(|(w, _)| w).collect()
    }

    /// 获取预定义规则
    pub fn default_rules() -> Vec<CategoryRule> {
        vec![
            CategoryRule {
                id: "rule-docs".to_string(),
                name: "文档".to_string(),
                description: "文档类文件".to_string(),
                target_folder: "文档".to_string(),
                extensions: vec!["doc".into(), "docx".into(), "pdf".into(), "txt".into(), "md".into(), "rtf".into(), "odt".into()],
                ..Default::default()
            },
            CategoryRule {
                id: "rule-images".to_string(),
                name: "图片".to_string(),
                description: "图片类文件".to_string(),
                target_folder: "图片".to_string(),
                extensions: vec!["jpg".into(), "jpeg".into(), "png".into(), "gif".into(), "bmp".into(), "svg".into(), "webp".into(), "ico".into()],
                ..Default::default()
            },
            CategoryRule {
                id: "rule-videos".to_string(),
                name: "视频".to_string(),
                description: "视频类文件".to_string(),
                target_folder: "视频".to_string(),
                extensions: vec!["mp4".into(), "avi".into(), "mkv".into(), "mov".into(), "wmv".into(), "flv".into(), "webm".into()],
                min_size: Some(1024 * 1024), // 1MB以上
                ..Default::default()
            },
            CategoryRule {
                id: "rule-audio".to_string(),
                name: "音频".to_string(),
                description: "音频类文件".to_string(),
                target_folder: "音频".to_string(),
                extensions: vec!["mp3".into(), "wav".into(), "flac".into(), "aac".into(), "ogg".into(), "wma".into(), "m4a".into()],
                ..Default::default()
            },
            CategoryRule {
                id: "rule-archives".to_string(),
                name: "压缩包".to_string(),
                description: "压缩文件".to_string(),
                target_folder: "压缩包".to_string(),
                extensions: vec!["zip".into(), "rar".into(), "7z".into(), "tar".into(), "gz".into(), "xz".into(), "bz2".into()],
                ..Default::default()
            },
            CategoryRule {
                id: "rule-code".to_string(),
                name: "代码".to_string(),
                description: "源代码文件".to_string(),
                target_folder: "代码".to_string(),
                extensions: vec![
                    "rs".into(), "py".into(), "js".into(), "ts".into(), "tsx".into(), "jsx".into(),
                    "java".into(), "c".into(), "cpp".into(), "h".into(), "hpp".into(), "cs".into(),
                    "go".into(), "rb".into(), "php".into(), "swift".into(), "kt".into(), "vue".into(),
                    "css".into(), "scss".into(), "less".into(), "html".into(), "json".into(), "xml".into(),
                ],
                ..Default::default()
            },
            CategoryRule {
                id: "rule-spreadsheets".to_string(),
                name: "表格".to_string(),
                description: "电子表格文件".to_string(),
                target_folder: "表格".to_string(),
                extensions: vec!["xls".into(), "xlsx".into(), "csv".into(), "ods".into()],
                ..Default::default()
            },
            CategoryRule {
                id: "rule-presentations".to_string(),
                name: "演示文稿".to_string(),
                description: "PPT类文件".to_string(),
                target_folder: "演示文稿".to_string(),
                extensions: vec!["ppt".into(), "pptx".into(), "odp".into()],
                ..Default::default()
            },
            CategoryRule {
                id: "rule-executables".to_string(),
                name: "可执行文件".to_string(),
                description: "程序和安装文件".to_string(),
                target_folder: "程序".to_string(),
                extensions: vec!["exe".into(), "msi".into(), "apk".into(), "dmg".into(), "app".into()],
                ..Default::default()
            },
            CategoryRule {
                id: "rule-disk-images".to_string(),
                name: "磁盘镜像".to_string(),
                description: "磁盘和系统镜像文件".to_string(),
                target_folder: "磁盘镜像".to_string(),
                extensions: vec!["iso".into(), "img".into(), "vhd".into(), "vhdx".into(), "dmg".into()],
                min_size: Some(10 * 1024 * 1024), // 10MB以上
                ..Default::default()
            },
        ]
    }
}

fn common_prefix_len(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(ca, cb)| ca == cb).count()
}

use std::time::UNIX_EPOCH;
