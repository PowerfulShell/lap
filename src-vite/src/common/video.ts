import { invoke } from '@tauri-apps/api/core';

export interface VideoPrepareResult {
  url: string;
  is_remuxed: boolean;
}

export async function prepareVideo(filePath: string): Promise<VideoPrepareResult> {
  return invoke<VideoPrepareResult>('prepare_video', { filePath });
}

export async function cancelVideoPrepare(): Promise<void> {
  return invoke('cancel_video_prepare');
}

export async function clearVideoCache(): Promise<void> {
  return invoke('clear_video_cache');
}
