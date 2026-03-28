import { invoke } from '@tauri-apps/api/core';
import { config } from '@/common/config';

let telemetryAvailable = true;

function getLocalDayBucket() {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

export function isTelemetryEnabled() {
  return !!config.settings.telemetry?.enabled;
}

export async function trackTelemetryEvent(name: string, props?: Record<string, string | number>) {
  if (!telemetryAvailable || !isTelemetryEnabled()) return;

  try {
    await invoke('plugin:aptabase|track_event', {
      name,
      props: props && Object.keys(props).length > 0 ? props : null,
    });
  } catch (error) {
    telemetryAvailable = false;
    console.debug('Telemetry unavailable:', error);
  }
}

export async function trackAppLifecycleEvents() {
  if (!isTelemetryEnabled()) return;

  const dayBucket = getLocalDayBucket();
  if (config.settings.telemetry.lastDailyActiveDate !== dayBucket) {
    config.settings.telemetry.lastDailyActiveDate = dayBucket;
    void trackTelemetryEvent('daily_active', { day: dayBucket });
  }

  void trackTelemetryEvent('app_started');
}
