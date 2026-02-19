# CategorizedFileList 公共组件使用指南

`CategorizedFileList` 是一个可复用的分类文件列表组件，支持懒加载、无限滚动、文件选择等功能。

## 基本用法

```tsx
import { CategorizedFileList } from '../components/common';
import type { BaseFileInfo, BaseCategoryInfo, FileRowProps } from '../components/common';

// 1. 定义文件类型（继承 BaseFileInfo）
interface MyFileInfo extends BaseFileInfo {
  customField: string;
}

// 2. 定义分类类型（继承 BaseCategoryInfo）
interface MyCategoryInfo extends BaseCategoryInfo<MyFileInfo> {
  customCategoryField: number;
}

// 3. 使用组件
function MyComponent() {
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());

  const categories: MyCategoryInfo[] = [
    {
      key: 'logs',
      displayName: '日志文件',
      files: [],  // 初始可为空，懒加载时填充
      fileCount: 100,
      totalSize: 1024000,
      hasMore: true,  // 关键：设置为 true 启用懒加载
    },
  ];

  return (
    <CategorizedFileList<MyFileInfo, MyCategoryInfo>
      categories={categories}
      selectedFiles={selectedFiles}
      expandedCategories={expandedCategories}
      getCategoryKey={(c) => c.key}
      getFileKey={(f) => f.path}
      onToggleFileSelection={(path) => { /* ... */ }}
      onToggleCategorySelection={(key) => { /* ... */ }}
      onToggleCategoryExpand={(key) => { /* ... */ }}
      onSelectAll={() => { /* ... */ }}
      onDeselectAll={() => { /* ... */ }}
      onExpandAll={() => { /* ... */ }}
      onCollapseAll={() => { /* ... */ }}
      totalCount={100}
      totalSize={1024000}
      selectedCount={0}
      selectedSize={0}
    />
  );
}
```

## 懒加载支持

### 1. 后端分页 API

首先需要在后端实现分页查询接口：

```rust
// Rust 后端示例
#[tauri::command]
pub async fn get_category_files(
    scan_id: String,
    category_name: String,
    offset: u64,
    limit: u64,
) -> Option<CategoryFilesResponse> {
    // 从存储中获取文件列表
    let files = get_files_from_store(&scan_id, &category_name)?;
    
    // 分页切片
    let start = offset as usize;
    let end = std::cmp::min(start + limit as usize, files.len());
    
    Some(CategoryFilesResponse {
        files: files[start..end].to_vec(),
        total: files.len() as u64,
        has_more: end < files.len(),
    })
}
```

### 2. 前端服务封装

```typescript
// services/scanService.ts
export const scanService = {
  getCategoryFiles: async (
    scanId: string,
    categoryName: string,
    offset: number,
    limit: number
  ): Promise<CategoryFilesResponse | null> => {
    return invoke<CategoryFilesResponse | null>('get_category_files', {
      scanId,
      categoryName,
      offset,
      limit,
    });
  },
};
```

### 3. 实现 onLoadMore 回调

```tsx
function MyComponent() {
  const scanId = 'scan-123';

  // 懒加载回调
  const handleLoadMore = useCallback(async (
    categoryKey: string,
    offset: number,
    limit: number
  ) => {
    try {
      const response = await scanService.getCategoryFiles(
        scanId,
        categoryKey,
        offset,
        limit
      );

      if (response) {
        // 转换为组件需要的文件格式
        const files: MyFileInfo[] = response.files.map(item => ({
          path: item.path,
          name: item.name,
          size: item.size,
          modified_time: item.modified_time,
          customField: item.custom_field,
        }));

        return {
          files,
          hasMore: response.has_more,
        };
      }
    } catch (error) {
      console.error('加载失败:', error);
    }
    return null;
  }, [scanId]);

  return (
    <CategorizedFileList
      // ... 其他属性
      onLoadMore={handleLoadMore}  // 传入懒加载回调
    />
  );
}
```

### 4. 设置 hasMore 标志

分类数据必须正确设置 `hasMore` 属性：

```tsx
const categories: MyCategoryInfo[] = results.map(result => ({
  key: result.category_key,
  displayName: result.display_name,
  files: result.initial_files,       // 初始加载的文件（可为空数组）
  fileCount: result.total_count,     // 总文件数
  totalSize: result.total_size,
  hasMore: result.total_count > result.initial_files.length,  // 关键判断
}));
```

## 无限滚动原理

组件内部使用 `IntersectionObserver` 监听底部元素：

```tsx
// CategoryItem.tsx 内部实现
useEffect(() => {
  if (!isExpanded || !hasMore || isLoading || !onLoadMore) return;

  const observer = new IntersectionObserver(
    (entries) => {
      if (entries[0].isIntersecting) {
        handleLoadMore();  // 触发加载
      }
    },
    { threshold: 0.1, rootMargin: '100px' }  // 提前100px触发
  );

  if (loadMoreRef.current) {
    observer.observe(loadMoreRef.current);
  }

  return () => observer.disconnect();
}, [isExpanded, hasMore, isLoading, handleLoadMore]);
```

## 完整示例

```tsx
import { CategorizedFileList } from '../components/common';
import type { BaseFileInfo, BaseCategoryInfo, FileRowProps } from '../components/common';
import { scanService } from '../services/scanService';

interface ScanFileInfo extends BaseFileInfo {
  category: string;
}

interface ScanCategoryInfo extends BaseCategoryInfo<ScanFileInfo> {
  description: string;
}

function ScanResults({ scanId, results }: Props) {
  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());

  // 转换分类数据
  const categories: ScanCategoryInfo[] = useMemo(() => 
    results.map(cat => ({
      key: cat.name,
      displayName: cat.display_name,
      description: cat.description,
      files: cat.files,
      fileCount: cat.file_count,
      totalSize: cat.total_size,
      hasMore: cat.has_more,
    })),
    [results]
  );

  // 懒加载
  const handleLoadMore = useCallback(async (categoryKey, offset, limit) => {
    const response = await scanService.getCategoryFiles(scanId, categoryKey, offset, limit);
    if (response) {
      return {
        files: response.files.map(f => ({ ...f, category: categoryKey })),
        hasMore: response.has_more,
      };
    }
    return null;
  }, [scanId]);

  // 选择管理
  const toggleFile = (path: string) => {
    const newSet = new Set(selectedFiles);
    newSet.has(path) ? newSet.delete(path) : newSet.add(path);
    setSelectedFiles(newSet);
  };

  const toggleCategory = (key: string) => {
    const category = categories.find(c => c.key === key);
    if (!category) return;
    const allSelected = category.files.every(f => selectedFiles.has(f.path));
    const newSet = new Set(selectedFiles);
    category.files.forEach(f => allSelected ? newSet.delete(f.path) : newSet.add(f.path));
    setSelectedFiles(newSet);
  };

  const toggleExpand = (key: string) => {
    const newSet = new Set(expandedCategories);
    newSet.has(key) ? newSet.delete(key) : newSet.add(key);
    setExpandedCategories(newSet);
  };

  return (
    <CategorizedFileList<ScanFileInfo, ScanCategoryInfo>
      categories={categories}
      selectedFiles={selectedFiles}
      expandedCategories={expandedCategories}
      getCategoryKey={c => c.key}
      getFileKey={f => f.path}
      onToggleFileSelection={toggleFile}
      onToggleCategorySelection={toggleCategory}
      onToggleCategoryExpand={toggleExpand}
      onSelectAll={() => setSelectedFiles(new Set(categories.flatMap(c => c.files.map(f => f.path))))}
      onDeselectAll={() => setSelectedFiles(new Set())}
      onExpandAll={() => setExpandedCategories(new Set(categories.map(c => c.key)))}
      onCollapseAll={() => setExpandedCategories(new Set())}
      onLoadMore={handleLoadMore}
      totalCount={results.reduce((s, c) => s + c.file_count, 0)}
      totalSize={results.reduce((s, c) => s + c.total_size, 0)}
      selectedCount={selectedFiles.size}
      selectedSize={Array.from(selectedFiles).reduce((s, p) => s + getFileSize(p), 0)}
    />
  );
}
```

## API 参考

### Props

| 属性 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `categories` | `TCategory[]` | ✅ | 分类数据数组 |
| `selectedFiles` | `Set<string>` | ✅ | 已选择的文件 key 集合 |
| `expandedCategories` | `Set<string>` | ✅ | 已展开的分类 key 集合 |
| `getCategoryKey` | `(c: TCategory) => string` | ✅ | 获取分类唯一标识 |
| `getFileKey` | `(f: TFile) => string` | ✅ | 获取文件唯一标识 |
| `onToggleFileSelection` | `(key: string) => void` | ✅ | 切换文件选择 |
| `onToggleCategorySelection` | `(key: string) => void` | ✅ | 切换分类选择 |
| `onToggleCategoryExpand` | `(key: string) => void` | ✅ | 切换分类展开 |
| `onSelectAll` | `() => void` | ✅ | 全选 |
| `onDeselectAll` | `() => void` | ✅ | 取消全选 |
| `onExpandAll` | `() => void` | ✅ | 全部展开 |
| `onCollapseAll` | `() => void` | ✅ | 全部折叠 |
| `totalCount` | `number` | ✅ | 文件总数 |
| `totalSize` | `number` | ✅ | 文件总大小 |
| `selectedCount` | `number` | ✅ | 已选择文件数 |
| `selectedSize` | `number` | ✅ | 已选择文件大小 |
| `onLoadMore` | `(key, offset, limit) => Promise<{files, hasMore} \| null>` | ❌ | 懒加载回调 |
| `onOpenLocation` | `(file: TFile) => void` | ❌ | 打开文件位置 |
| `renderFileRow` | `(props: FileRowProps) => ReactNode` | ❌ | 自定义文件行渲染 |
| `maxListHeight` | `number` | ❌ | 列表最大高度（默认 300） |
| `loadMoreLimit` | `number` | ❌ | 每次加载数量（默认 50） |

### BaseFileInfo

```typescript
interface BaseFileInfo {
  path: string;
  name?: string;
  size: number;
  modified_time?: number;
}
```

### BaseCategoryInfo

```typescript
interface BaseCategoryInfo<TFile extends BaseFileInfo = BaseFileInfo> {
  key: string;
  displayName: string;
  description?: string;
  files: TFile[];
  fileCount: number;
  totalSize: number;
  hasMore?: boolean;  // 设为 true 启用懒加载
  icon?: ReactNode;
  iconColor?: string;
}
```

## 注意事项

1. **hasMore 必须正确设置**：只有当 `hasMore: true` 时，无限滚动才会生效
2. **fileCount vs files.length**：`fileCount` 是总数，`files.length` 是当前已加载数
3. **内存管理**：后端需要存储完整文件列表，前端按需加载
4. **错误处理**：`onLoadMore` 返回 `null` 时会停止加载
5. **性能优化**：使用 `useCallback` 包装回调函数避免不必要的重渲染
