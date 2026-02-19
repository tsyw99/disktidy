import type { ReactNode } from 'react';

export interface BaseFileInfo {
  path: string;
  name?: string;
  size: number;
  modified_time?: number;
}

export interface BaseCategoryInfo<TFile extends BaseFileInfo = BaseFileInfo> {
  key: string;
  displayName: string;
  description?: string;
  files: TFile[];
  fileCount: number;
  totalSize: number;
  hasMore?: boolean;
  icon?: ReactNode;
  iconColor?: string;
}

export interface CategoryFilesState<TFile extends BaseFileInfo = BaseFileInfo> {
  files: TFile[];
  total: number;
  hasMore: boolean;
  isLoading: boolean;
}

export interface FileRowProps<TFile extends BaseFileInfo = BaseFileInfo> {
  file: TFile;
  isSelected: boolean;
  isLast: boolean;
  onToggleSelection: () => void;
  onOpenLocation?: (file: TFile) => void;
}

export interface CategoryItemProps<TFile extends BaseFileInfo = BaseFileInfo> {
  category: BaseCategoryInfo<TFile>;
  isExpanded: boolean;
  isSelected: boolean;
  selectedFiles: Set<string>;
  filesState: CategoryFilesState<TFile>;
  onToggleExpand: () => void;
  onToggleSelection: () => void;
  onToggleFileSelection: (filePath: string) => void;
  onLoadMore?: () => void;
  renderFileRow?: (props: FileRowProps<TFile>) => ReactNode;
  onOpenLocation?: (file: TFile) => void;
  getFileKey: (file: TFile) => string;
}

export interface CategorizedFileListProps<TFile extends BaseFileInfo = BaseFileInfo, TCategory extends BaseCategoryInfo<TFile> = BaseCategoryInfo<TFile>> {
  categories: TCategory[];
  selectedFiles: Set<string>;
  expandedCategories: Set<string>;
  
  getCategoryKey: (category: TCategory) => string;
  getFileKey: (file: TFile) => string;
  
  onToggleFileSelection: (fileKey: string) => void;
  onToggleCategorySelection: (categoryKey: string) => void;
  onToggleCategoryExpand: (categoryKey: string) => void;
  onSelectAll: () => void;
  onDeselectAll: () => void;
  onExpandAll: () => void;
  onCollapseAll: () => void;
  
  onLoadMore?: (categoryKey: string, offset: number, limit: number) => Promise<{ files: TFile[]; hasMore: boolean } | null>;
  onOpenLocation?: (file: TFile) => void;
  
  totalCount: number;
  totalSize: number;
  selectedCount: number;
  selectedSize: number;
  
  renderFileRow?: (props: FileRowProps<TFile>) => ReactNode;
  renderCategoryHeader?: (category: TCategory) => ReactNode;
  
  emptyText?: string;
  showExpandCollapse?: boolean;
  showSelectAll?: boolean;
  maxListHeight?: number;
  loadMoreLimit?: number;
  
  title?: string;
  showHelpButton?: boolean;
  onHelpClick?: () => void;
}

export const DEFAULT_LOAD_MORE_LIMIT = 50;
export const DEFAULT_MAX_LIST_HEIGHT = 300;
