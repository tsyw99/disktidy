import { invoke } from '@tauri-apps/api/core';
import type { FileClassificationResult, FileClassificationOptions } from '../types';

/**
 * 优化的文件分类配置 - 完全独立于磁盘扫描设置
 * 这些配置是固定的，不受用户系统设置影响
 */
export const OPTIMIZED_CLASSIFICATION_CONFIG: FileClassificationOptions = {
  max_depth: undefined,
  include_hidden: true,
  include_system: false,
  exclude_paths: [
    '$Recycle.Bin',
    'System Volume Information',
    'Config.Msi',
    'MSOCache',
    'pagefile.sys',
    'hiberfil.sys',
    'swapfile.sys',
  ],
  max_files: undefined,
  top_n_categories: 20,
};

/**
 * 图例识别验证结果
 */
export interface LegendValidationResult {
  isValid: boolean;
  confidence: number;
  recognizedCategories: string[];
  unrecognizedItems: string[];
  suggestions: string[];
}

/**
 * 分类统计验证器
 */
export class ClassificationValidator {
  /**
   * 验证分类结果的完整性和准确性
   */
  static validateClassification(result: FileClassificationResult): LegendValidationResult {
    const recognizedCategories: string[] = [];
    const unrecognizedItems: string[] = [];
    const suggestions: string[] = [];
    
    // 预定义的标准文件类型
    const standardCategories = [
      '视频', '音频', '图片', '压缩包', '可执行文件',
      '磁盘镜像', '文档', '数据库', '文本文件', '配置文件',
      '系统文件', '代码文件', '字体文件', '备份文件', '临时文件', '其他'
    ];
    
    // 验证每个分类
    for (const category of result.categories) {
      if (standardCategories.includes(category.category)) {
        recognizedCategories.push(category.category);
        
        // 验证数据合理性
        if (category.count === 0) {
          suggestions.push(`分类 "${category.category}" 文件数量为0，建议检查扫描范围`);
        }
        if (category.total_size === 0 && category.count > 0) {
          suggestions.push(`分类 "${category.category}" 文件大小为0，可能存在空文件`);
        }
        if (category.percentage > 100 || category.percentage < 0) {
          suggestions.push(`分类 "${category.category}" 百分比数据异常: ${category.percentage}%`);
        }
      } else {
        unrecognizedItems.push(category.category);
      }
    }
    
    // 计算置信度
    const totalCategories = result.categories.length;
    const recognizedCount = recognizedCategories.length;
    const confidence = totalCategories > 0 ? recognizedCount / totalCategories : 0;
    
    // 验证总数据一致性
    const calculatedTotal = result.categories.reduce((sum, cat) => sum + cat.total_size, 0);
    if (Math.abs(calculatedTotal - result.total_size) > 1024) {
      suggestions.push('分类总大小与扫描总大小存在差异，可能存在数据不一致');
    }
    
    return {
      isValid: confidence > 0.8 && unrecognizedItems.length === 0,
      confidence,
      recognizedCategories,
      unrecognizedItems,
      suggestions,
    };
  }
  
  /**
   * 检查分类结果是否需要重新扫描
   */
  static shouldRescan(result: FileClassificationResult): boolean {
    const validation = this.validateClassification(result);
    
    // 置信度低于阈值需要重扫
    if (validation.confidence < 0.7) return true;
    
    // 存在未识别分类需要重扫
    if (validation.unrecognizedItems.length > 0) return true;
    
    // 文件数量异常需要重扫
    if (result.total_files === 0 && result.total_folders === 0) return true;
    
    return false;
  }
}

/**
 * 优化的文件分类服务
 * 完全独立于系统磁盘扫描设置
 */
export const optimizedFileClassifier = {
  /**
   * 执行文件分类 - 使用固定配置，不受系统设置影响
   */
  classifyFiles: async (
    path: string, 
    customOptions?: Partial<FileClassificationOptions>
  ): Promise<FileClassificationResult> => {
    // 合并配置，但优先使用固定配置，确保不受系统设置影响
    const options: FileClassificationOptions = {
      ...OPTIMIZED_CLASSIFICATION_CONFIG,
      ...customOptions,
      // 强制覆盖可能影响结果的设置
      include_hidden: customOptions?.include_hidden ?? OPTIMIZED_CLASSIFICATION_CONFIG.include_hidden,
      include_system: false, // 始终排除系统文件以保证安全
    };
    
    const result = await invoke<FileClassificationResult>('classify_files', { 
      path, 
      options 
    });
    
    // 验证结果
    const validation = ClassificationValidator.validateClassification(result);
    
    // 如果结果无效，记录警告但不阻止返回
    if (!validation.isValid) {
      console.warn('文件分类结果验证警告:', validation.suggestions);
    }
    
    return result;
  },

  /**
   * 分类磁盘 - 使用优化配置
   */
  classifyDisk: async (disk: string): Promise<FileClassificationResult> => {
    const path = disk === 'all' ? 'C:\\' : `${disk.replace(/\\+$/, '')}\\`;
    
    return optimizedFileClassifier.classifyFiles(path, OPTIMIZED_CLASSIFICATION_CONFIG);
  },

  /**
   * 开始分类（支持取消）- 使用优化配置
   */
  startClassifyFiles: async (
    path: string, 
    customOptions?: Partial<FileClassificationOptions>
  ): Promise<FileClassificationResult> => {
    const options: FileClassificationOptions = {
      ...OPTIMIZED_CLASSIFICATION_CONFIG,
      ...customOptions,
      include_hidden: customOptions?.include_hidden ?? OPTIMIZED_CLASSIFICATION_CONFIG.include_hidden,
      include_system: false,
    };
    
    const result = await invoke<FileClassificationResult>('start_classify_files', { 
      path, 
      options 
    });
    
    return result;
  },

  /**
   * 取消分类
   */
  cancelClassifyFiles: (): Promise<boolean> =>
    invoke<boolean>('cancel_classify_files'),

  /**
   * 验证分类结果
   */
  validateResult: (result: FileClassificationResult): LegendValidationResult => {
    return ClassificationValidator.validateClassification(result);
  },

  /**
   * 获取分类统计摘要
   */
  getClassificationSummary: (result: FileClassificationResult) => {
    const validation = ClassificationValidator.validateClassification(result);
    
    return {
      totalCategories: result.categories.length,
      totalFiles: result.total_files,
      totalSize: result.total_size,
      avgFileSize: result.total_files > 0 ? result.total_size / result.total_files : 0,
      largestCategory: result.categories[0]?.category || 'N/A',
      confidence: validation.confidence,
      isReliable: validation.isValid,
      scanDuration: result.duration_ms,
    };
  },
};

export default optimizedFileClassifier;
