<template>
  <div ref="videoContainer" class="relative w-full h-full overflow-hidden cursor-pointer bg-black" @wheel.prevent="handleWheel">
    <TransitionGroup :name="transitionName" @after-leave="handleTransitionEnd">
      <div
        v-for="index in [0, 1]"
        v-show="activeLayer === index"
        :key="`video-layer-${index}`"
        class="slide-wrapper absolute inset-0 w-full h-full pointer-events-none overflow-hidden"
      >
        <div class="w-full h-full pointer-events-auto overflow-hidden flex items-center justify-center">
          <!-- The video.js target -->
          <div 
            v-if="videoUrl && (activeLayer === index || isTransitioning)"
            :id="`vjs-container-${index}`"
            class="vjs-custom-container"
            :class="{ 'no-transition': noTransition }"
            :style="surfaceStyle"
          >
            <video
              :id="`vjs-player-${index}`"
              class="video-js vjs-big-play-centered vjs-fluid"
            ></video>
          </div>
        </div>
      </div>
    </TransitionGroup>

    <!-- Loading Overlay -->
    <div v-if="isPreparing" class="absolute inset-0 flex items-center justify-center bg-black/40 z-20">
      <div class="flex flex-col items-center gap-3">
        <span class="loading loading-spinner loading-lg text-primary"></span>
        <span class="text-white/70 text-sm font-medium">{{ $t('video.preparing') || 'Preparing video...' }}</span>
      </div>
    </div>

    <!-- Error Overlay -->
    <div v-if="hasError" class="absolute inset-0 flex flex-col items-center justify-center text-base-content/30 z-10 bg-black/20">
      <IconVideoSlash class="w-12 h-12 mb-3" />
      <div class="text-center px-6 max-w-md">{{ errorMessage }}</div>
      <div class="text-center px-6 max-w-md mt-2 text-sm text-white/60 whitespace-pre-line">
        {{ externalSuggestionText }}
      </div>
      <div class="mt-4 flex items-center gap-2">
        <button class="btn btn-outline btn-sm" @click="loadVideo(props.filePath!)">Retry</button>
        <button v-if="canOpenExternalApp" class="btn btn-primary btn-sm" @click="openInExternalApp">
          {{ externalOpenLabel }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch, nextTick } from 'vue';
import { convertFileSrc } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import videojs from 'video.js';
import 'video.js/dist/video-js.css';

import { config } from '@/common/config';
import { IconVideoSlash } from '@/common/icons';
import { cancelVideoPrepare, prepareVideo } from '@/common/video';
import { openFileWithApp } from '@/common/api';

const props = defineProps({
  filePath: { type: String, required: false },
  rotate: { type: Number, default: 0 },
  isZoomFit: { type: Boolean, default: false },
  isSlideShow: { type: Boolean, default: false },
});

const emit = defineEmits(['message-from-video-viewer', 'slideshow-next', 'scale', 'viewport-change']);
const { t: $t } = useI18n();

const videoContainer = ref<HTMLDivElement | null>(null);
const players = ref<(ReturnType<typeof videojs> | null)[]>([]);

const hasError = ref(false);
const errorMessage = ref('');
const isPlaying = ref(false);
const isPreparing = ref(false);
const videoUrl = ref<string | null>(null);

const scale = ref(1);
const rotate = ref(0);
const noTransition = ref(false);
const activeLayer = ref(0);
const isTransitioning = ref(false);
const isFit = ref(false);

let currentLoadingId = 0;

// Gesture state
let isTouchpadDevice = false;
let horizontalDeltaAccumulator = 0;
let verticalDeltaAccumulator = 0;
let gestureResetTimeout: ReturnType<typeof setTimeout> | null = null;
let hasNavigatedThisGesture = false;
const gestureType = ref<'none' | 'zoom' | 'nav'>('none');
const navDirection = ref<'next' | 'prev' | ''>('');
const GESTURE_LOCK_THRESHOLD = 10;
const HORIZONTAL_NAV_THRESHOLD = 100;

const transitionName = computed(() => {
  if (props.isSlideShow) return 'slide-next';
  if (navDirection.value) return navDirection.value === 'next' ? 'slide-next' : 'slide-prev';
  return '';
});

const externalVideoAppPath = computed(() => String(config.settings.externalVideoAppPath || '').trim());
const externalVideoAppName = computed(() => String(config.settings.externalVideoAppName || '').trim());
const canOpenExternalApp = computed(() => !!externalVideoAppPath.value);
const externalSuggestionText = computed(() => {
  if (canOpenExternalApp.value) {
    return $t('video.errors.use_external_with_app');
  }
  return $t('video.errors.use_external');
});
const externalOpenLabel = computed(() => {
  if (externalVideoAppName.value) {
    return $t('video.errors.open_in_external_app_named', { app: externalVideoAppName.value });
  }
  return $t('video.errors.open_in_external_app');
});

const surfaceStyle = computed(() => ({
  transform: `rotate(${rotate.value}deg) scale(${scale.value})`,
}));

const handleTransitionEnd = () => {
  navDirection.value = '';
  isTransitioning.value = false;
};

const disposePlayer = (index: number) => {
  if (players.value[index]) {
    players.value[index]?.dispose();
    players.value[index] = null;
  }
};

const initPlayer = (index: number, url: string) => {
  disposePlayer(index);
  
  const playerId = `vjs-player-${index}`;
  const p = videojs(playerId, {
    autoplay: config.settings.autoPlayVideo || props.isSlideShow,
    controls: true,
    fluid: true,
    responsive: true,
    muted: config.video.muted,
    volume: config.video.volume,
    sources: [{ src: convertFileSrc(url), type: 'video/mp4' }],
    userActions: { doubleClick: true }
  });

  p.on('play', () => isPlaying.value = true);
  p.on('pause', () => isPlaying.value = false);
  p.on('ended', () => {
    isPlaying.value = false;
    if (props.isSlideShow) emit('slideshow-next');
  });
  p.on('error', () => {
    hasError.value = true;
    errorMessage.value = $t('video.errors.playback_failed') || 'Playback failed';
  });
  p.on('volumechange', () => {
    config.video.volume = p.volume() || 0;
    config.video.muted = p.muted() || false;
  });

  players.value[index] = p;
};

async function openInExternalApp() {
  if (!props.filePath || !externalVideoAppPath.value) return;
  await openFileWithApp(props.filePath, externalVideoAppPath.value);
}

async function loadVideo(filePath: string) {
  currentLoadingId += 1;
  const loadingId = currentLoadingId;
  
  isPreparing.value = true;
  hasError.value = false;
  errorMessage.value = '';
  isPlaying.value = false;

  try {
    const result = await prepareVideo(filePath);
    if (loadingId !== currentLoadingId) return;

    videoUrl.value = result.url;
    isTransitioning.value = true;
    activeLayer.value ^= 1;
    
    // Dispose the OTHER layer's player
    disposePlayer(activeLayer.value ^ 1);

    await nextTick();
    initPlayer(activeLayer.value, result.url);

    noTransition.value = true;
    isFit.value = props.isZoomFit;
    scale.value = 1;
    rotate.value = props.rotate;
    emitViewportChange();

    setTimeout(() => {
      noTransition.value = false;
      isPreparing.value = false;
    }, 100);
  } catch (error: any) {
    if (loadingId !== currentLoadingId) return;
    isPreparing.value = false;
    hasError.value = true;
    errorMessage.value = error?.message || String(error);
  }
}

const emitViewportChange = () => {
  emit('scale', { scale: scale.value, minScale: 0.1, maxScale: 10 });
  emit('viewport-change', { scale: scale.value, isZoomFit: isFit.value, fileType: 2 });
};

defineExpose({
  zoomIn: () => { scale.value = Math.min(scale.value * 2, 10); emitViewportChange(); },
  zoomOut: () => { scale.value = Math.max(scale.value / 2, 0.1); emitViewportChange(); },
  zoomActual: () => { scale.value = 1; emitViewportChange(); },
  rotateRight: () => { rotate.value = (rotate.value + 90) % 360; emitViewportChange(); },
  togglePlay: () => {
    const p = players.value[activeLayer.value];
    if (p) p.paused() ? p.play() : p.pause();
  },
  pause: () => players.value[activeLayer.value]?.pause(),
});

onMounted(() => { if (props.filePath) loadVideo(props.filePath); });
onBeforeUnmount(() => {
  players.value.forEach((_, i) => disposePlayer(i));
  cancelVideoPrepare();
});

watch(() => props.filePath, (v) => v && loadVideo(v));
watch(() => props.rotate, (v) => rotate.value = v);

function handleWheel(event: WheelEvent) {
  event.preventDefault();
  if (event.deltaX !== 0) isTouchpadDevice = true;
  if (gestureResetTimeout) clearTimeout(gestureResetTimeout);
  gestureResetTimeout = setTimeout(() => {
    gestureType.value = 'none'; horizontalDeltaAccumulator = 0; verticalDeltaAccumulator = 0; hasNavigatedThisGesture = false;
  }, 150);

  if (isTouchpadDevice) {
    if (gestureType.value === 'none') {
      horizontalDeltaAccumulator += event.deltaX;
      verticalDeltaAccumulator += event.deltaY;
      const absX = Math.abs(horizontalDeltaAccumulator);
      const absY = Math.abs(verticalDeltaAccumulator);
      if (absX > GESTURE_LOCK_THRESHOLD || absY > GESTURE_LOCK_THRESHOLD) gestureType.value = absX > absY ? 'nav' : 'zoom';
    } else if (gestureType.value === 'nav') {
      horizontalDeltaAccumulator += event.deltaX;
      if (!hasNavigatedThisGesture && Math.abs(horizontalDeltaAccumulator) >= HORIZONTAL_NAV_THRESHOLD) {
        const direction = horizontalDeltaAccumulator > 0 ? 'next' : 'prev';
        navDirection.value = direction;
        emit('message-from-video-viewer', { message: direction });
        hasNavigatedThisGesture = true;
      }
    } else {
      const delta = -event.deltaY * 0.01;
      scale.value = Math.max(0.1, Math.min(10, scale.value + delta));
      emitViewportChange();
    }
  } else {
    const zoomFactor = 0.1;
    scale.value = event.deltaY < 0 ? Math.min(scale.value * (1 + zoomFactor), 10) : Math.max(scale.value * (1 - zoomFactor), 0.1);
    emitViewportChange();
  }
}
</script>

<style scoped>
.vjs-custom-container {
  width: 100%;
  max-width: 100%;
  max-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform 0.3s ease-out;
}
.vjs-custom-container.no-transition {
  transition: none;
}

:deep(.video-js) {
  background-color: transparent;
}
:deep(.vjs-tech) {
  object-fit: contain;
}

.slide-next-enter-active, .slide-next-leave-active { transition: transform 0.6s ease-in-out; }
.slide-next-enter-from { transform: translateX(100%); }
.slide-next-leave-to { transform: translateX(-100%); }
.slide-prev-enter-from { transform: translateX(-100%); }
.slide-prev-leave-to { transform: translateX(100%); }
</style>
