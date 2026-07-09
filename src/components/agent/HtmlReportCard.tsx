import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { motion, AnimatePresence } from 'framer-motion';
import { FileBarChart, Maximize2, Minimize2, ExternalLink, FolderOpen, CheckCircle2, AlertTriangle } from 'lucide-react';

interface HtmlReportCardProps {
  htmlContent?: string;
  toolName?: string;
  /** 文件路径（generate_html 工具输出，文件已保存在磁盘） */
  filePath?: string;
}

/**
 * 从任何格式的字符串中提取纯 HTML 内容
 */
function extractHtml(raw: string): string {
  const trimmed = raw.trim();

  if (trimmed.startsWith('<!DOCTYPE') || trimmed.startsWith('<html')) {
    return trimmed;
  }

  const doctypeIdx = trimmed.indexOf('<!DOCTYPE html>');
  if (doctypeIdx >= 0) {
    const tail = trimmed.slice(doctypeIdx);
    const endIdx = tail.indexOf('</html>');
    if (endIdx >= 0) {
      return tail.slice(0, endIdx + 7)
        .replace(/\\"/g, '"')
        .replace(/\\n/g, '\n')
        .replace(/\\t/g, '\t')
        .replace(/\\u003c/g, '<')
        .replace(/\\u003e/g, '>');
    }
  }

  const htmlIdx = trimmed.indexOf('<html');
  if (htmlIdx >= 0) {
    const tail = trimmed.slice(htmlIdx);
    const endIdx = tail.indexOf('</html>');
    if (endIdx >= 0) {
      return tail.slice(0, endIdx + 7)
        .replace(/\\"/g, '"')
        .replace(/\\n/g, '\n')
        .replace(/\\t/g, '\t')
        .replace(/\\u003c/g, '<')
        .replace(/\\u003e/g, '>');
    }
  }

  return trimmed;
}

/** 从路径中提取文件名（去除扩展名作为显示标题） */
function fileNameFromPath(filePath: string): string {
  const parts = filePath.replace(/\\/g, '/').split('/');
  return parts[parts.length - 1] || filePath;
}

function dirFromPath(filePath: string): string {
  const parts = filePath.replace(/\\/g, '/').split('/');
  parts.pop();
  return parts.join('\\') || filePath;
}

export default function HtmlReportCard({ htmlContent, toolName, filePath }: HtmlReportCardProps) {
  const [showPreview, setShowPreview] = useState(false);
  const [isExpanded, setIsExpanded] = useState(false);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const [statusError, setStatusError] = useState(false);

  const showStatus = (msg: string, isError = false) => {
    setStatusMsg(msg);
    setStatusError(isError);
    setTimeout(() => setStatusMsg(null), 3000);
  };

  /** 在浏览器中打开已保存的 HTML 文件 */
  const handleOpenFileInBrowser = async () => {
    if (!filePath) return;
    try {
      await invoke('system_open_file', { path: filePath });
      showStatus('已在浏览器中打开');
    } catch (e) {
      showStatus(`打开失败: ${e}`, true);
    }
  };

  /** 在资源管理器中打开文件所在目录 */
  const handleOpenFolder = async () => {
    if (!filePath) return;
    try {
      await invoke('system_open_folder', { path: filePath });
      showStatus('已打开文件所在目录');
    } catch (e) {
      showStatus(`打开失败: ${e}`, true);
    }
  };

  /** 打开嵌入 HTML 内容到浏览器（旧工作流） */
  const handleOpenHtmlContent = async () => {
    if (!htmlContent) return;
    try {
      await invoke('system_open_in_browser', {
        request: { html_content: extractHtml(htmlContent) },
      });
      showStatus('已在浏览器中打开');
    } catch (e) {
      showStatus(`打开失败: ${e}`, true);
    }
  };

  // ====== 模式 1: 有文件路径（generate_html 工作流） ======
  if (filePath) {
    const displayName = fileNameFromPath(filePath);
    const dirPath = dirFromPath(filePath);

    return (
      <div className="mt-2">
        <motion.div
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          className="rounded-xl border border-gray-200 dark:border-gray-600 bg-white dark:bg-gray-800 overflow-hidden shadow-sm hover:shadow-md transition-shadow"
        >
          {/* 主区域 — 点击整张卡片打开文件 */}
          <button
            onClick={handleOpenFileInBrowser}
            className="w-full text-left flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors group"
          >
            <div className="w-10 h-10 rounded-lg bg-indigo-100 dark:bg-indigo-900/30 flex items-center justify-center flex-shrink-0 group-hover:bg-indigo-200 dark:group-hover:bg-indigo-800/40 transition-colors">
              <FileBarChart className="w-5 h-5 text-indigo-500" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-sm font-medium text-gray-800 dark:text-gray-200 truncate">
                {displayName}
              </div>
              <div className="text-xs text-gray-400 dark:text-gray-500 truncate mt-0.5" title={filePath}>
                {dirPath}
              </div>
            </div>
            <ExternalLink className="w-4 h-4 text-gray-300 dark:text-gray-500 group-hover:text-indigo-500 transition-colors flex-shrink-0" />
          </button>

          {/* 底栏 — 打开目录按钮 */}
          <div className="flex items-center justify-end px-3 py-1.5 border-t border-gray-100 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50">
            <button
              onClick={handleOpenFolder}
              className="flex items-center gap-1 text-xs text-gray-400 dark:text-gray-500 hover:text-indigo-500 dark:hover:text-indigo-400 transition-colors py-1 px-2 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
              title="打开文件所在目录"
            >
              <FolderOpen className="w-3.5 h-3.5" />
              <span>打开目录</span>
            </button>
          </div>

          {/* 状态提示 */}
          <AnimatePresence>
            {statusMsg && (
              <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: 'auto', opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                className="overflow-hidden"
              >
                <div className={`flex items-center gap-1.5 px-4 py-1.5 text-xs ${statusError ? 'text-red-600 bg-red-50' : 'text-green-600 bg-green-50'}`}>
                  {statusError ? <AlertTriangle className="w-3 h-3 flex-shrink-0" /> : <CheckCircle2 className="w-3 h-3 flex-shrink-0" />}
                  <span className="truncate">{statusMsg}</span>
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </motion.div>
      </div>
    );
  }

  // ====== 模式 2: 嵌入 HTML 内容（旧 file_content_analyzer 工作流） ======
  const html = htmlContent ? extractHtml(htmlContent) : '';

  return (
    <div className="mt-2">
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        className="rounded-xl border-2 border-indigo-400/30 bg-indigo-50 dark:bg-indigo-900/10 overflow-hidden"
      >
        {/* 卡片头部 */}
        <div className="flex items-center justify-between px-4 py-2.5 bg-indigo-100/50 dark:bg-indigo-800/20">
          <div className="flex items-center gap-2 min-w-0">
            <FileBarChart className="w-4 h-4 text-indigo-500 flex-shrink-0" />
            <span className="text-xs font-medium text-indigo-700 dark:text-indigo-300 truncate">
              {toolName === 'file_content_analyzer' ? '文件内容分析报告' : '可视化报告'}
            </span>
          </div>
          <div className="flex items-center gap-1 flex-shrink-0">
            <button
              onClick={() => setShowPreview(!showPreview)}
              className="p-1.5 rounded hover:bg-indigo-200/50 dark:hover:bg-indigo-700/30 text-indigo-500 transition-colors"
              title={showPreview ? '收起预览' : '展开预览'}
            >
              {showPreview ? <Minimize2 className="w-3.5 h-3.5" /> : <Maximize2 className="w-3.5 h-3.5" />}
            </button>
            <button
              onClick={handleOpenHtmlContent}
              className="p-1.5 rounded hover:bg-indigo-200/50 dark:hover:bg-indigo-700/30 text-indigo-500 transition-colors"
              title="在默认浏览器中打开"
            >
              <ExternalLink className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>

        {/* 状态提示 */}
        <AnimatePresence>
          {statusMsg && (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: 'auto', opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              className="overflow-hidden"
            >
              <div className={`flex items-center gap-1.5 px-4 py-1.5 text-xs ${statusError ? 'text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/10' : 'text-green-600 dark:text-green-400 bg-green-50 dark:bg-green-900/10'}`}>
                {statusError ? <AlertTriangle className="w-3 h-3 flex-shrink-0" /> : <CheckCircle2 className="w-3 h-3 flex-shrink-0" />}
                <span className="truncate">{statusMsg}</span>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* iframe 预览 */}
        <AnimatePresence>
          {showPreview && (
            <motion.div
              initial={{ height: 0 }}
              animate={{ height: isExpanded ? '75vh' : '420px' }}
              exit={{ height: 0 }}
              transition={{ duration: 0.3 }}
              className="overflow-hidden border-t border-indigo-200/30 dark:border-indigo-700/30"
            >
              <div className="relative w-full h-full">
                <iframe
                  srcDoc={html}
                  className="w-full h-full border-0 bg-white"
                  sandbox="allow-scripts allow-same-origin"
                  title="HTML 报告预览"
                />
                <button
                  onClick={() => setIsExpanded(!isExpanded)}
                  className="absolute top-2 right-2 p-1.5 rounded-lg bg-white/90 dark:bg-gray-800/90 shadow hover:shadow-md transition-shadow backdrop-blur-sm"
                  title={isExpanded ? '缩小预览' : '扩大预览'}
                >
                  {isExpanded ? (
                    <Minimize2 className="w-4 h-4 text-gray-600 dark:text-gray-300" />
                  ) : (
                    <Maximize2 className="w-4 h-4 text-gray-600 dark:text-gray-300" />
                  )}
                </button>
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
}
