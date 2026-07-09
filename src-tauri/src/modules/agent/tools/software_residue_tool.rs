use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::modules::software_residue::scanner::{SoftwareResidueScanner, ResidueScanOptions};
use super::tool_error::ToolError;
use super::ToolPrompt;

#[derive(Debug, Deserialize)]
pub struct SoftwareResidueToolInput {
    #[serde(default)]
    pub scan_all_drives: bool,
}

#[derive(Debug, Serialize)]
pub struct SoftwareResidueToolOutput {
    pub scan_id: String,
    pub total_residues: u64,
    pub total_size: u64,
    pub residues: Vec<serde_json::Value>,
    pub _prompt: String,
}

impl ToolPrompt for SoftwareResidueTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 软件残留扫描工作流
你正在使用 software_residue_scanner 工具扫描已卸载软件的残留文件，请遵循以下流程：

1. 展示扫描结果：发现的残留项数量和总大小
2. 用表格展示：残留路径 | 类型（文件夹/注册表/文件）| 大小 | 关联的软件
3. 分析风险：
   - 空目录 → 安全删除
   - 配置文件 → 谨慎处理（可能被其他软件共享）
   - 缓存数据 → 可安全清理
4. 建议调用 cleaner 工具清理确认的残留项
"#
    }
}

pub struct SoftwareResidueTool {
    #[allow(dead_code)]
    app: AppHandle,
}

impl SoftwareResidueTool {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Tool for SoftwareResidueTool {
    const NAME: &'static str = "software_residue_scanner";
    type Error = ToolError;
    type Args = SoftwareResidueToolInput;
    type Output = SoftwareResidueToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "扫描已卸载软件的残留文件。返回残留文件列表及其大小。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "scan_all_drives": {
                        "type": "boolean",
                        "description": "是否扫描所有磁盘驱动器，默认为 false"
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let options = ResidueScanOptions {
            scan_all_drives: args.scan_all_drives,
            ..Default::default()
        };

        let scanner = SoftwareResidueScanner::with_options(options);
        let results = scanner.start_scan().await.map_err(|e| ToolError::ScanFailed(e))?;

        // 聚合所有结果
        let total_residues = results.iter().map(|r| r.count).sum();
        let total_size = results.iter().map(|r| r.total_size).sum();

        Ok(SoftwareResidueToolOutput {
            scan_id: uuid::Uuid::new_v4().to_string(),
            total_residues,
            total_size,
            residues: results.iter().map(|r| serde_json::to_value(r).unwrap_or_default()).collect(),
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}
