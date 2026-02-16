import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener';

export async function openFileLocation(filePath: string): Promise<void> {
  try {
    await revealItemInDir(filePath);
  } catch (error) {
    console.error('Failed to reveal file in directory:', error);
    try {
      const path = filePath.substring(0, filePath.lastIndexOf('\\'));
      await openPath(path);
    } catch (e) {
      console.error('Failed to open folder:', e);
    }
  }
}

export async function openFolder(folderPath: string): Promise<void> {
  try {
    await openPath(folderPath);
  } catch (error) {
    console.error('Failed to open folder:', error);
  }
}
