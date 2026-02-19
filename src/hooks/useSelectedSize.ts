import { useMemo } from 'react';

export interface SelectableItem {
  path: string;
  size: number;
}

export function useSelectedSize<T extends SelectableItem>(
  items: T[],
  selectedFiles: Set<string>
): number {
  return useMemo(() => {
    let size = 0;
    for (const item of items) {
      if (selectedFiles.has(item.path)) {
        size += item.size;
      }
    }
    return size;
  }, [items, selectedFiles]);
}

export function useSelectedSizeFromResults<T extends SelectableItem>(
  results: { items: T[] }[],
  selectedFiles: Set<string>
): number {
  return useMemo(() => {
    let size = 0;
    for (const result of results) {
      for (const item of result.items) {
        if (selectedFiles.has(item.path)) {
          size += item.size;
        }
      }
    }
    return size;
  }, [results, selectedFiles]);
}
