use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;

use super::reader::FileContent;
use super::reader::FileFormat;

/// 关键词项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordItem {
    pub word: String,
    pub count: usize,
    pub score: f64,
}

/// 内容结构章节
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureSection {
    pub level: usize,
    pub title: String,
    pub line_start: usize,
    pub line_end: usize,
    pub char_count: usize,
}

/// 内容分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    pub file_path: String,
    pub file_name: String,
    pub file_format: String,
    pub file_size_bytes: u64,
    pub total_chars: usize,
    pub total_lines: usize,
    pub total_words: usize,
    pub unique_words: usize,
    pub avg_word_length: f64,
    pub paragraphs: usize,
    pub sentences: usize,
    pub reading_time_minutes: f64,
    pub top_keywords: Vec<KeywordItem>,
    pub structure: Vec<StructureSection>,
    pub language_hint: String,
    pub content_density: f64,
}

/// 停用词列表（中英文常见停用词）
const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
    "have", "has", "had", "do", "does", "did", "will", "would", "could",
    "should", "may", "might", "can", "shall", "must", "need", "dare",
    "of", "in", "to", "for", "with", "on", "at", "from", "by", "about",
    "as", "into", "through", "during", "before", "after", "above", "below",
    "between", "out", "off", "over", "under", "again", "further", "then",
    "once", "here", "there", "when", "where", "why", "how", "all", "both",
    "each", "few", "more", "most", "other", "some", "such", "no", "nor",
    "not", "only", "own", "same", "so", "than", "too", "very", "just",
    "and", "but", "or", "if", "while", "that", "this", "it", "its",
    "he", "she", "they", "them", "their", "his", "her", "my", "your",
    "our", "me", "us", "we", "you", "i", "him", "who", "whom", "which",
    "what", "these", "those", "am", "also",
    // 中文停用词
    "的", "了", "在", "是", "我", "有", "和", "就", "不", "人", "都", "一",
    "一个", "上", "也", "很", "到", "说", "要", "去", "你", "会", "着",
    "没有", "看", "好", "自己", "这", "他", "她", "它", "们", "那", "些",
    "所", "为", "所以", "因为", "但是", "然而", "如果", "虽然", "可以",
    "这个", "那个", "什么", "怎么", "如何", "为什么", "已经", "还是",
    "只是", "而且", "然后", "不过", "吧", "呢", "吗", "啊", "哦", "嗯",
];

pub struct ContentAnalyzer;

impl ContentAnalyzer {
    /// 分析文件内容
    pub fn analyze(file_content: &FileContent) -> ContentAnalysis {
        let content = &file_content.content;
        let words: Vec<&str> = Self::tokenize_words(content);
        let total_words = words.len();
        let unique_words = Self::count_unique(&words);
        let total_chars_alpha: usize = words.iter().map(|w| w.chars().count()).sum();
        let avg_word_length = if total_words > 0 {
            total_chars_alpha as f64 / total_words as f64
        } else {
            0.0
        };
        let paragraphs = Self::count_paragraphs(content);
        let sentences = Self::count_sentences(content);
        let reading_time = total_words as f64 / 200.0; // 平均阅读速度200词/分钟
        let top_keywords = Self::extract_keywords(content, &words, 20);
        let structure = Self::detect_structure(content, &file_content.format);
        let language_hint = Self::detect_language(&words);
        let content_density = if file_content.size_bytes > 0 {
            (content.len() as f64 / file_content.size_bytes as f64).min(1.0)
        } else {
            0.0
        };

        ContentAnalysis {
            file_path: file_content.path.clone(),
            file_name: file_content.name.clone(),
            file_format: file_content.format.to_string(),
            file_size_bytes: file_content.size_bytes,
            total_chars: file_content.char_count,
            total_lines: file_content.line_count,
            total_words,
            unique_words,
            avg_word_length,
            paragraphs,
            sentences,
            reading_time_minutes: reading_time,
            top_keywords,
            structure,
            language_hint,
            content_density,
        }
    }

    /// 对比两个文件的内容相似度
    pub fn similarity(a: &ContentAnalysis, b: &ContentAnalysis) -> f64 {
        let keywords_a: HashMap<&str, f64> = a.top_keywords.iter()
            .map(|k| (k.word.as_str(), k.score))
            .collect();
        let keywords_b: HashMap<&str, f64> = b.top_keywords.iter()
            .map(|k| (k.word.as_str(), k.score))
            .collect();

        let all_words: Vec<&str> = keywords_a.keys()
            .chain(keywords_b.keys())
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        if all_words.is_empty() {
            return 0.0;
        }

        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for word in &all_words {
            let va = keywords_a.get(word).copied().unwrap_or(0.0);
            let vb = keywords_b.get(word).copied().unwrap_or(0.0);
            dot_product += va * vb;
            norm_a += va * va;
            norm_b += vb * vb;
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a.sqrt() * norm_b.sqrt())
    }

    fn tokenize_words(content: &str) -> Vec<&str> {
        content.split_word_bounds()
            .filter(|w| {
                let trimmed = w.trim();
                !trimmed.is_empty() &&
                trimmed.chars().any(|c| c.is_alphanumeric()) &&
                trimmed.len() >= 2
            })
            .collect()
    }

    fn count_unique(words: &[&str]) -> usize {
        let set: std::collections::HashSet<&&str> = words.iter().collect();
        set.len()
    }

    fn count_paragraphs(content: &str) -> usize {
        content.split("\n\n").filter(|p| !p.trim().is_empty()).count().max(1)
    }

    fn count_sentences(content: &str) -> usize {
        content.split(|c: char| c == '.' || c == '!' || c == '?' || c == '。' || c == '！' || c == '？')
            .filter(|s| !s.trim().is_empty())
            .count()
            .max(1)
    }

    fn extract_keywords(_content: &str, words: &[&str], top_n: usize) -> Vec<KeywordItem> {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for word in words {
            let lower = word.to_lowercase();
            if STOP_WORDS.contains(&lower.as_str()) {
                continue;
            }
            *freq.entry(lower).or_insert(0) += 1;
        }

        let total = freq.values().sum::<usize>() as f64;
        let mut items: Vec<KeywordItem> = freq.into_iter()
            .filter(|(_, count)| *count >= 2)
            .map(|(word, count)| {
                let tf = count as f64 / total.max(1.0);
                KeywordItem {
                    word,
                    count,
                    score: tf * 100.0,
                }
            })
            .collect();

        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        items.truncate(top_n);
        items
    }

    fn detect_structure(content: &str, format: &FileFormat) -> Vec<StructureSection> {
        let mut sections = Vec::new();

        match format {
            FileFormat::Markdown => {
                // 提取 Markdown 标题结构
                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('#') {
                        let level = trimmed.chars().take_while(|c| *c == '#').count();
                        let title = trimmed[level..].trim().to_string();
                        if level <= 4 {
                            sections.push(StructureSection {
                                level,
                                title,
                                line_start: line_num + 1,
                                line_end: line_num + 1,
                                char_count: 0,
                            });
                        }
                    }
                }
            }
            FileFormat::Json => {
                // JSON 顶层键作为结构
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
                    if let Some(obj) = value.as_object() {
                        for (key, _) in obj {
                            sections.push(StructureSection {
                                level: 1,
                                title: key.clone(),
                                line_start: 0,
                                line_end: 0,
                                char_count: 0,
                            });
                        }
                    } else if let Some(arr) = value.as_array() {
                        sections.push(StructureSection {
                            level: 1,
                            title: format!("数组 ({} 项)", arr.len()),
                            line_start: 0,
                            line_end: 0,
                            char_count: 0,
                        });
                    }
                }
            }
            FileFormat::Xml => {
                // 简化XML结构提取
                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with('<') && !trimmed.starts_with("</") && !trimmed.starts_with("<?") {
                        let tag_end = trimmed.find(|c: char| c == '>' || c == ' ').unwrap_or(trimmed.len());
                        let tag = &trimmed[1..tag_end];
                        if !tag.is_empty() {
                            sections.push(StructureSection {
                                level: 1,
                                title: tag.to_string(),
                                line_start: line_num + 1,
                                line_end: line_num + 1,
                                char_count: 0,
                            });
                        }
                    }
                }
            }
            _ => {
                // 通用：检测疑似标题行（全大写、带编号等）
                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() || trimmed.len() < 3 {
                        continue;
                    }
                    // 检测带编号的标题
                    let starts_with_number = trimmed.chars().next().map(|c| c.is_numeric()).unwrap_or(false);
                    let is_all_uppercase = trimmed.chars().all(|c| c.is_uppercase() || !c.is_alphabetic());
                    let is_short_line = trimmed.len() <= 60 && !trimmed.ends_with('。') && !trimmed.ends_with('.');

                    if is_short_line && (is_all_uppercase || starts_with_number) {
                        sections.push(StructureSection {
                            level: 1,
                            title: trimmed.to_string(),
                            line_start: line_num + 1,
                            line_end: line_num + 1,
                            char_count: trimmed.chars().count(),
                        });
                    }
                }
            }
        }

        sections
    }

    fn detect_language(words: &[&str]) -> String {
        let mut chinese_count = 0;
        let mut english_count = 0;

        for word in words {
            let has_chinese = word.chars().any(|c| c as u32 >= 0x4E00 && c as u32 <= 0x9FFF);
            let has_english = word.chars().any(|c| c.is_ascii_alphabetic());

            if has_chinese { chinese_count += 1; }
            if has_english { english_count += 1; }
        }

        if chinese_count > english_count * 2 {
            "中文".to_string()
        } else if english_count > chinese_count * 2 {
            "English".to_string()
        } else if chinese_count > 0 && english_count > 0 {
            "中英混合".to_string()
        } else {
            "未知".to_string()
        }
    }
}
