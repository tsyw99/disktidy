import { useMemo } from 'react';

interface MarkdownRendererProps {
  content: string;
  className?: string;
}

/**
 * 轻量级 Markdown 渲染器
 *
 * 支持：标题、粗体、斜体、代码块、行内代码、
 * 列表、表格、链接、分隔线
 */
export default function MarkdownRenderer({ content, className = '' }: MarkdownRendererProps) {
  const html = useMemo(() => renderMarkdown(content), [content]);

  return (
    <div
      className={`markdown-body ${className}`}
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

function renderMarkdown(text: string): string {
  // 先按块分割
  const lines = text.split('\n');
  const blocks: string[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    // 代码块 ```
    if (line.trim().startsWith('```')) {
      const lang = line.trim().slice(3).trim();
      const codeLines: string[] = [];
      i++;
      while (i < lines.length && !lines[i].trim().startsWith('```')) {
        codeLines.push(escapeHtml(lines[i]));
        i++;
      }
      i++; // skip closing ```
      const langClass = lang ? ` class="language-${lang}"` : '';
      blocks.push(
        `<pre><code${langClass}>${codeLines.join('\n')}</code></pre>`
      );
      continue;
    }

    // 表格
    if (line.includes('|') && line.trim().startsWith('|')) {
      const tableLines: string[] = [line];
      i++;
      while (i < lines.length && lines[i].includes('|') && lines[i].trim().startsWith('|')) {
        tableLines.push(lines[i]);
        i++;
      }
      blocks.push(renderTable(tableLines));
      continue;
    }

    // 标题
    const headingMatch = line.match(/^(#{1,6})\s+(.+)/);
    if (headingMatch) {
      const level = headingMatch[1].length;
      const text = headingMatch[2];
      blocks.push(`<h${level} class="md-heading md-h${level}">${renderInline(text)}</h${level}>`);
      i++;
      continue;
    }

    // 分隔线
    if (/^[-*_]{3,}\s*$/.test(line.trim())) {
      blocks.push('<hr class="md-hr" />');
      i++;
      continue;
    }

    // 无序列表
    if (/^\s*[-*+]\s+/.test(line)) {
      const listItems: string[] = [];
      while (i < lines.length && /^\s*[-*+]\s+/.test(lines[i])) {
        const itemText = lines[i].replace(/^\s*[-*+]\s+/, '');
        listItems.push(`<li>${renderInline(itemText)}</li>`);
        i++;
      }
      blocks.push(`<ul class="md-list">${listItems.join('')}</ul>`);
      continue;
    }

    // 有序列表
    if (/^\s*\d+\.\s+/.test(line)) {
      const listItems: string[] = [];
      while (i < lines.length && /^\s*\d+\.\s+/.test(lines[i])) {
        const itemText = lines[i].replace(/^\s*\d+\.\s+/, '');
        listItems.push(`<li>${renderInline(itemText)}</li>`);
        i++;
      }
      blocks.push(`<ol class="md-list md-ordered">${listItems.join('')}</ol>`);
      continue;
    }

    // 普通段落
    if (line.trim() === '') {
      i++;
      continue;
    }

    // 合并连贯的文本行
    const paraLines: string[] = [];
    while (i < lines.length && lines[i].trim() !== '' && !isBlockStart(lines[i])) {
      paraLines.push(lines[i]);
      i++;
    }
    const paraText = paraLines.join('\n');
    blocks.push(`<p class="md-paragraph">${renderInline(paraText)}</p>`);
  }

  return blocks.join('\n');
}

function isBlockStart(line: string): boolean {
  return (
    /^#{1,6}\s/.test(line) ||
    /^[-*_]{3,}\s*$/.test(line.trim()) ||
    line.trim().startsWith('```') ||
    /^\s*[-*+]\s/.test(line) ||
    /^\s*\d+\.\s/.test(line) ||
    (line.includes('|') && line.trim().startsWith('|'))
  );
}

function renderTable(lines: string[]): string {
  if (lines.length < 2) return '';

  // 过滤掉分隔行（|---|---|）
  const dataLines = lines.filter((_, idx) => {
    if (idx === 0) return true;
    // 分隔行形如 |---|---|
    return !/^\|[\s\-:|]+\|$/.test(lines[idx]);
  });

  const headerCells = dataLines[0]
    .split('|')
    .filter((c) => c.trim() !== '')
    .map((c) => `<th>${renderInline(c.trim())}</th>`)
    .join('');

  const bodyRows = dataLines
    .slice(1)
    .map((row) => {
      const cells = row
        .split('|')
        .filter((c) => c.trim() !== '')
        .map((c) => `<td>${renderInline(c.trim())}</td>`)
        .join('');
      return `<tr>${cells}</tr>`;
    })
    .join('');

  return `<table class="md-table"><thead><tr>${headerCells}</tr></thead><tbody>${bodyRows}</tbody></table>`;
}

function renderInline(text: string): string {
  // 先转义 HTML
  let result = escapeHtml(text);

  // 粗体 **text**
  result = result.replace(/\*\*(.+?)\*\*/g, '<strong class="md-bold">$1</strong>');

  // 斜体 *text*
  result = result.replace(/\*(.+?)\*/g, '<em class="md-italic">$1</em>');

  // 行内代码 `text`
  result = result.replace(/`([^`]+)`/g, '<code class="md-inline-code">$1</code>');

  // 链接 [text](url)
  result = result.replace(
    /\[([^\]]+)\]\(([^)]+)\)/g,
    '<a href="$2" class="md-link" target="_blank" rel="noopener">$1</a>'
  );

  return result;
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}