<template>
  <ModalDialog :title="`${$t('msgbox.image_editor.adjust_title')} - ${shortenFilename(props.fileInfo.name, 32)}`" :width="1040" @cancel="clickCancel">
    <div class="h-[560px] flex gap-4 select-none">
      <div class="flex-1 min-w-0 h-full flex items-start">
        <div class="relative w-full aspect-4/3 max-h-full rounded-box overflow-hidden border border-base-content/5 bg-base-300/30 shadow-sm cursor-default">
          <transition name="fade">
            <div v-if="isProcessing" class="absolute inset-0 z-50 flex items-center justify-center bg-base-100/55 backdrop-blur-sm">
              <span class="loading loading-dots text-primary"></span>
            </div>
          </transition>

          <img :src="imageSrc" :style="imageStyle" class="w-full h-full object-contain" draggable="false" @load="onImageLoad" />
        </div>
      </div>

      <div class="w-[268px] flex flex-col gap-3 overflow-y-auto">
        <section class="rounded-box p-3 space-y-2 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="flex items-center justify-between gap-2">
            <div class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.histogram') }}</div>
          </div>

          <div class="relative w-full aspect-4/1 px-0.5">
            <svg viewBox="0 0 256 64" class="w-full h-full text-primary" preserveAspectRatio="none">
              <defs>
                <linearGradient id="histGradientAdjust" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="currentColor" stop-opacity="0.6" />
                  <stop offset="100%" stop-color="currentColor" stop-opacity="0.1" />
                </linearGradient>
              </defs>
              <g class="text-base-content/20">
                <line x1="64" y1="0" x2="64" y2="64" stroke="currentColor" stroke-width="0.5" />
                <line x1="128" y1="0" x2="128" y2="64" stroke="currentColor" stroke-width="0.5" />
                <line x1="192" y1="0" x2="192" y2="64" stroke="currentColor" stroke-width="0.5" />
              </g>

              <path :d="generateHistogramPath()" fill="url(#histGradientAdjust)" class="transition-all duration-300" />
            </svg>
          </div>

          <div class="flex justify-between px-0.5 text-[8px] uppercase tracking-tighter font-black text-base-content/25">
            <span>{{ $t('msgbox.image_editor.shadows') || 'Shadows' }}</span>
            <span>{{ $t('msgbox.image_editor.midtones') || 'Midtones' }}</span>
            <span>{{ $t('msgbox.image_editor.highlights') || 'Highlights' }}</span>
          </div>
        </section>

        <section class="rounded-box p-3 space-y-2 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-2 min-w-0">
              <IconPalette class="w-4 h-4 text-base-content/60" />
              <span class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.presets.title') }}</span>
            </div>
            <span class="text-[10px] text-primary/70 font-bold uppercase mr-1">{{ presetOptions.find((o) => o.value === selectedPreset)?.label || selectedPreset }}</span>
            <TButton
              buttonSize="small"
              :icon="IconRestore"
              :disabled="!hasAdjustmentChanges"
              :tooltip="$t('msgbox.image_editor.reset')"
              @click.stop="resetAll"
            />
          </div>

          <div ref="presetStripRef" class="flex gap-2 overflow-x-auto overflow-y-hidden flex-nowrap">
            <div
              v-for="option in presetOptions"
              :key="option.value"
              :data-preset="option.value"
              class="shrink-0 w-[76px] group cursor-pointer"
              @click="selectedPreset = option.value"
            >
              <div
                :class="[
                  'aspect-4/3 rounded-box border-2 transition-all duration-200 flex items-center justify-center overflow-hidden mb-1 relative',
                  selectedPreset === option.value ? 'border-primary ring-2 ring-primary/20' : 'border-base-content/5 hover:border-base-content/20',
                ]"
              >
                <div class="w-full h-full bg-base-300 flex items-center justify-center relative overflow-hidden rounded-[inherit] isolation-isolate">
                  <img
                    v-if="props.fileInfo.thumbnail"
                    :src="props.fileInfo.thumbnail"
                    class="w-full h-full object-cover pointer-events-none rounded-[inherit] block"
                    :style="{
                      ...getPresetThumbnailStyle(option.value),
                      transform: 'translateZ(0)',
                    }"
                  />
                  <IconPalette v-else class="w-4 h-4 text-base-content/10" />
                </div>
              </div>
              <div class="text-[9px] text-center truncate font-medium text-base-content/50 group-hover:text-base-content transition-colors uppercase tracking-tight">
                {{ option.label }}
              </div>
            </div>
          </div>
        </section>

        <section class="rounded-box p-3 space-y-2 border border-base-content/5 shadow-sm bg-base-300/30">
          <div class="flex items-center justify-between gap-2">
            <div class="flex items-center gap-2">
              <IconAdjustments class="w-4 h-4 text-base-content/60" />
              <span class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.adjustments') }}</span>
            </div>
            <TButton
              buttonSize="small"
              :icon="IconRestore"
              :disabled="!hasAdjustmentChanges"
              :tooltip="$t('msgbox.image_editor.reset')"
              @click.stop="resetAll"
            />
          </div>

          <div class="space-y-4 overflow-hidden">
            <div class="space-y-3">
              <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/20 mb-2">{{ $t('msgbox.image_editor.light') || 'Light' }}</div>
              <div v-for="adj in lightSliders" :key="adj.key" class="grid grid-cols-[80px_minmax(0,1fr)] gap-x-4 items-center">
                <div class="font-medium text-base-content/40 tracking-wide text-xs">{{ adj.label }}</div>
                <div class="flex items-center gap-2 pr-2 min-w-0">
                  <SliderInput v-model="adj.model.value" :min="adj.min" :max="adj.max" :step="adj.step" class="flex-1 min-w-0 w-full" />
                  <span class="text-[10px] font-mono text-base-content/60 w-6 text-right shrink-0">{{ adj.valueDisplay }}</span>
                </div>
              </div>
            </div>

            <div class="h-px bg-base-content/5 mx-1"></div>

            <div class="space-y-3">
              <div class="text-[10px] uppercase tracking-widest font-bold text-base-content/20 mb-2">{{ $t('msgbox.image_editor.color') || 'Color' }}</div>
              <div v-for="adj in colorSliders" :key="adj.key" class="grid grid-cols-[80px_minmax(0,1fr)] gap-x-4 items-center">
                <div class="font-medium text-base-content/40 tracking-wide text-xs">{{ adj.label }}</div>
                <div class="flex items-center gap-2 pr-2 min-w-0">
                  <SliderInput v-model="adj.model.value" :min="adj.min" :max="adj.max" :step="adj.step" class="flex-1 min-w-0 w-full" />
                  <span class="text-[10px] font-mono text-base-content/60 w-6 text-right shrink-0">{{ adj.valueDisplay }}</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        <section class="rounded-box p-3 space-y-3 bg-base-300/30 border border-base-content/5 shadow-sm">
          <div class="text-[11px] font-bold uppercase tracking-[0.22em] text-base-content/35">{{ $t('msgbox.image_editor.save_file') }}</div>

          <div class="space-y-3">
            <div class="form-control w-full">
              <select v-model="config.imageEditor.saveAs" class="select select-bordered select-sm w-full">
                <option v-for="option in fileSaveAsOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
              </select>
            </div>

            <div v-if="config.imageEditor.saveAs !== 0" class="grid grid-cols-2 gap-2">
              <div class="form-control w-full">
                <label class="label py-1">
                  <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.format') }}</span>
                </label>
                <select v-model="config.imageEditor.format" class="select select-bordered select-sm w-full">
                  <option v-for="option in fileFormatOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
                </select>
              </div>

              <div v-if="config.imageEditor.format == 0" class="form-control w-full">
                <label class="label py-1">
                  <span class="label-text text-xs font-medium opacity-70">{{ $t('msgbox.image_editor.quality') }}</span>
                </label>
                <select v-model="config.imageEditor.quality" class="select select-bordered select-sm w-full">
                  <option v-for="option in fileQualityOptions" :key="option.value" :value="option.value">{{ option.label }}</option>
                </select>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>

    <div class="mt-1 flex justify-end space-x-4">
      <button class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer" @click="clickCancel">{{ $t('msgbox.image_editor.cancel') }}</button>
      <button
        :class="[
          'px-4 py-1 rounded-box',
          isProcessing ? 'text-base-content/30 cursor-default' : 'hover:bg-primary hover:text-base-100 cursor-pointer',
        ]"
        @click="clickSave"
      >
        {{ config.imageEditor.saveAs === 1 ? $t('msgbox.image_editor.save_as_new') : $t('msgbox.image_editor.overwrite') }}
      </button>
    </div>
  </ModalDialog>

  <MessageBox
    v-if="showOverwriteConfirm"
    :title="$t('msgbox.image_editor.overwrite')"
    :message="$t('msgbox.image_editor.overwrite_confirm')"
    :warningOk="true"
    :OkText="$t('msgbox.ok')"
    :cancelText="$t('msgbox.cancel')"
    @ok="handleOverwriteConfirm"
    @cancel="handleOverwriteCancel"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch, type CSSProperties } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import { useI18n } from 'vue-i18n';
import { config } from '@/common/config';
import { getFolderPath, shortenFilename, getFullPath, combineFileName, getSelectOptions, getFileExtension, getAssetSrc } from '@/common/utils';
import { editImage, checkFileExists } from '@/common/api';

import ModalDialog from '@/components/ModalDialog.vue';
import MessageBox from '@/components/MessageBox.vue';
import TButton from '@/components/TButton.vue';
import SliderInput from '@/components/SliderInput.vue';

import {
  IconClose,
  IconRestore,
  IconPalette,
  IconAdjustments,
} from '@/common/icons';

const props = defineProps({
  fileInfo: {
    type: Object,
    required: true,
  },
});

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);

const uiStore = useUIStore();
const emit = defineEmits(['success', 'failed', 'cancel']);

const isProcessing = ref(false);
const imageSrc = ref('');
const showOverwriteConfirm = ref(false);

const brightness = ref(0);
const contrast = ref(0);
const saturation = ref(100);
const hue = ref(0);
const blur = ref(0);
const selectedFilter = ref('');
const selectedPreset = ref('natural');
const presetStripRef = ref<HTMLElement | null>(null);

let isApplyingPreset = false;

const presets: Record<string, any> = {
  natural: { brightness: 0, contrast: 0, saturation: 100, hue: 0, blur: 0, filter: '' },
  vivid: { brightness: 0, contrast: 10, saturation: 120, hue: 0, blur: 0, filter: '' },
  muted: { brightness: 0, contrast: -10, saturation: 80, hue: 0, blur: 0, filter: '' },
  warm: { brightness: 5, contrast: 0, saturation: 100, hue: 5, blur: 0, filter: '' },
  cool: { brightness: 5, contrast: 0, saturation: 100, hue: -5, blur: 0, filter: '' },
  bw: { brightness: 0, contrast: 0, saturation: 0, hue: 0, blur: 0, filter: 'grayscale' },
  vintage: { brightness: 10, contrast: -10, saturation: 60, hue: 0, blur: 0, filter: 'sepia' },
  invert: { brightness: 0, contrast: 0, saturation: 100, hue: 0, blur: 0, filter: 'invert' },
  kodak: { brightness: 10, contrast: 15, saturation: 120, hue: -5, blur: 0, filter: '' },
  toyo: { brightness: 5, contrast: 0, saturation: 110, hue: 5, blur: 0, filter: '' },
  cinematic: { brightness: 0, contrast: 20, saturation: 80, hue: 0, blur: 0, filter: '' },
  dramatic: { brightness: 0, contrast: 30, saturation: 110, hue: 0, blur: 0, filter: '' },
  cyberpunk: { brightness: 10, contrast: 20, saturation: 130, hue: -15, blur: 0, filter: '' },
};

const imageStyle = computed((): CSSProperties => ({
  display: 'block',
  filter: `
    brightness(${100 + brightness.value}%)
    contrast(${100 + contrast.value}%)
    blur(${blur.value}px)
    hue-rotate(${hue.value}deg)
    saturate(${saturation.value}%)
    ${selectedFilter.value === 'grayscale' ? 'grayscale(100%)' : ''}
    ${selectedFilter.value === 'sepia' ? 'sepia(100%)' : ''}
    ${selectedFilter.value === 'invert' ? 'invert(100%)' : ''}
  `,
}));

const histogramData = ref<number[]>(new Array(256).fill(0));

const updateRealHistogram = () => {
  if (!props.fileInfo?.thumbnail) return;

  const img = new Image();
  img.crossOrigin = 'anonymous';
  img.src = props.fileInfo.thumbnail;

  img.onload = () => {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) return;

    const size = 256;
    canvas.width = size;
    canvas.height = size;
    ctx.drawImage(img, 0, 0, size, size);

    const imageData = ctx.getImageData(0, 0, size, size).data;
    const hist = new Array(256).fill(0);

    for (let i = 0; i < imageData.length; i += 4) {
      const r = imageData[i];
      const g = imageData[i + 1];
      const b = imageData[i + 2];
      const gray = Math.round(0.2126 * r + 0.7152 * g + 0.0722 * b);
      hist[gray]++;
    }

    const maxVal = Math.max(...hist);
    histogramData.value = maxVal > 0 ? hist.map((v) => (v / maxVal) * 58) : new Array(256).fill(0);
  };
};

const generateHistogramPath = () => {
  if (!histogramData.value) return '';

  const width = 256;
  const height = 64;
  const br = (100 + brightness.value) / 100;
  const ct = (100 + contrast.value) / 100;

  const sampledPoints: { x: number; y: number }[] = [];
  const step = 2;

  for (let i = 0; i <= 256; i += step) {
    let sum = 0;
    let count = 0;
    const windowSize = 2;

    for (let j = Math.max(0, i - windowSize); j < Math.min(256, i + windowSize); j++) {
      sum += histogramData.value[j];
      count++;
    }

    const val = count > 0 ? sum / count : 0;
    const x = (i * br - 128) * ct + 128;
    const y = height - val;

    if (x >= -10 && x <= width + 10) sampledPoints.push({ x, y });
  }

  if (sampledPoints.length < 2) return '';

  let path = `M 0,${height}`;

  for (let i = 0; i < sampledPoints.length; i++) {
    const p = sampledPoints[i];
    if (i === 0) {
      path += ` L ${p.x.toFixed(1)},${p.y.toFixed(1)}`;
    } else {
      const prev = sampledPoints[i - 1];
      const cp1x = prev.x + (p.x - prev.x) / 2;
      const cp1y = prev.y;
      const cp2x = prev.x + (p.x - prev.x) / 2;
      const cp2y = p.y;
      path += ` C ${cp1x.toFixed(1)},${cp1y.toFixed(1)} ${cp2x.toFixed(1)},${cp2y.toFixed(1)} ${p.x.toFixed(1)},${p.y.toFixed(1)}`;
    }
  }

  path += ` L ${width},${height} Z`;
  return path;
};

const presetOptions = computed(() => [
  { value: 'custom', label: localeMsg.value.msgbox.image_editor.presets.custom },
  { value: 'natural', label: localeMsg.value.msgbox.image_editor.presets.natural },
  { value: 'vivid', label: localeMsg.value.msgbox.image_editor.presets.vivid },
  { value: 'muted', label: localeMsg.value.msgbox.image_editor.presets.muted },
  { value: 'warm', label: localeMsg.value.msgbox.image_editor.presets.warm },
  { value: 'cool', label: localeMsg.value.msgbox.image_editor.presets.cool },
  { value: 'bw', label: localeMsg.value.msgbox.image_editor.presets.bw },
  { value: 'vintage', label: localeMsg.value.msgbox.image_editor.presets.vintage },
  { value: 'kodak', label: localeMsg.value.msgbox.image_editor.presets.kodak },
  { value: 'toyo', label: localeMsg.value.msgbox.image_editor.presets.toyo },
  { value: 'cinematic', label: localeMsg.value.msgbox.image_editor.presets.cinematic },
  { value: 'dramatic', label: localeMsg.value.msgbox.image_editor.presets.dramatic },
  { value: 'cyberpunk', label: localeMsg.value.msgbox.image_editor.presets.cyberpunk },
  { value: 'invert', label: localeMsg.value.msgbox.image_editor.presets.invert },
]);

const movePresetSelection = (step: number) => {
  const options = presetOptions.value;
  if (!options.length) return;

  const currentIndex = options.findIndex((option) => option.value === selectedPreset.value);
  const safeIndex = currentIndex >= 0 ? currentIndex : 0;
  const nextIndex = (safeIndex + step + options.length) % options.length;
  selectedPreset.value = options[nextIndex].value;
};

const scrollSelectedPresetIntoView = () => {
  const strip = presetStripRef.value;
  const el = strip?.querySelector(`[data-preset="${selectedPreset.value}"]`) as HTMLElement | null;
  if (!strip || !el) return;
  el.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
};

const getPresetThumbnailStyle = (presetKey: string) => {
  if (presetKey === 'custom') return imageStyle.value;
  const p = presets[presetKey];
  if (!p) return {};

  return {
    filter: `
      brightness(${100 + p.brightness}%)
      contrast(${100 + p.contrast}%)
      blur(${p.blur}px)
      hue-rotate(${p.hue}deg)
      saturate(${p.saturation}%)
      ${p.filter === 'grayscale' ? 'grayscale(100%)' : ''}
      ${p.filter === 'sepia' ? 'sepia(100%)' : ''}
      ${p.filter === 'invert' ? 'invert(100%)' : ''}
    `,
  };
};

const lightSliders = computed(() => [
  {
    key: 'brightness',
    label: localeMsg.value.msgbox.image_editor.brightness,
    model: brightness,
    min: -100,
    max: 100,
    step: 1,
    valueDisplay: `${brightness.value > 0 ? '+' : ''}${brightness.value}`,
  },
  {
    key: 'contrast',
    label: localeMsg.value.msgbox.image_editor.contrast,
    model: contrast,
    min: -100,
    max: 100,
    step: 1,
    valueDisplay: `${contrast.value > 0 ? '+' : ''}${contrast.value}`,
  },
]);

const colorSliders = computed(() => [
  {
    key: 'saturation',
    label: localeMsg.value.msgbox.image_editor.saturation,
    model: saturation,
    min: 0,
    max: 200,
    step: 1,
    valueDisplay: `${saturation.value}%`,
  },
  {
    key: 'hue',
    label: localeMsg.value.msgbox.image_editor.hue_rotate,
    model: hue,
    min: -180,
    max: 180,
    step: 1,
    valueDisplay: `${hue.value}°`,
  },
]);

const hasAdjustmentChanges = computed(() => {
  const p = presets.natural;
  return (
    brightness.value !== p.brightness ||
    contrast.value !== p.contrast ||
    saturation.value !== p.saturation ||
    hue.value !== p.hue ||
    blur.value !== p.blur ||
    selectedFilter.value !== p.filter
  );
});

watch(selectedPreset, (newVal) => {
  if (newVal === 'custom') return;
  const p = presets[newVal];
  if (!p) return;

  isApplyingPreset = true;
  brightness.value = p.brightness;
  contrast.value = p.contrast;
  saturation.value = p.saturation;
  hue.value = p.hue;
  blur.value = p.blur;
  selectedFilter.value = p.filter;

  nextTick(() => {
    isApplyingPreset = false;
    scrollSelectedPresetIntoView();
  });
});

watch(selectedPreset, () => {
  nextTick(() => {
    scrollSelectedPresetIntoView();
  });
});

watch([brightness, contrast, saturation, hue, blur, selectedFilter], () => {
  if (isApplyingPreset) return;

  if (selectedPreset.value !== 'custom') {
    const p = presets[selectedPreset.value];
    if (
      p &&
      (brightness.value !== p.brightness ||
        contrast.value !== p.contrast ||
        saturation.value !== p.saturation ||
        hue.value !== p.hue ||
        blur.value !== p.blur ||
        selectedFilter.value !== p.filter)
    ) {
      selectedPreset.value = 'custom';
    }
  }

  uiStore.setActiveAdjustments(props.fileInfo.file_path, {
    brightness: brightness.value,
    contrast: contrast.value,
    saturation: saturation.value,
    hue: hue.value,
    blur: blur.value,
    filter: selectedFilter.value || null,
    rotate: 0,
    flipX: false,
    flipY: false,
    resize: null,
  });
});

const resetAll = () => {
  const p = presets.natural;
  brightness.value = p.brightness;
  contrast.value = p.contrast;
  saturation.value = p.saturation;
  hue.value = p.hue;
  blur.value = p.blur;
  selectedFilter.value = p.filter;
  selectedPreset.value = 'natural';
  if (uiStore.activeAdjustments.filePath === props.fileInfo.file_path) {
    uiStore.clearActiveAdjustments();
  }
};

const newFileName = ref(props.fileInfo.name.substring(0, props.fileInfo.name.lastIndexOf('.')) || props.fileInfo.name);
const fileSaveAsOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.save_as_options));
const fileFormatOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.format_options));
const fileQualityOptions = computed(() => getSelectOptions(localeMsg.value.msgbox.image_editor.quality_options));

const handleOverwriteConfirm = () => {
  showOverwriteConfirm.value = false;

  const originalPath = props.fileInfo.file_path;
  const ext = getFileExtension(props.fileInfo.name).toLowerCase();
  const outputFormat = ext === 'jpg' || ext === 'jpeg' ? 'jpg' : ext;

  executeSave({
    destFilePath: originalPath,
    outputFormat,
  });
};

const handleOverwriteCancel = () => {
  showOverwriteConfirm.value = false;
  isProcessing.value = false;
};

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('AdjustImage');

  isProcessing.value = true;
  imageSrc.value = getAssetSrc(props.fileInfo.file_path);
  updateRealHistogram();

  if (uiStore.activeAdjustments.filePath === props.fileInfo.file_path) {
    const adj = uiStore.activeAdjustments;
    brightness.value = adj.brightness || 0;
    contrast.value = adj.contrast || 0;
    saturation.value = adj.saturation ?? 100;
    hue.value = adj.hue || 0;
    blur.value = adj.blur || 0;
    selectedFilter.value = adj.filter || '';
    selectedPreset.value = 'custom';
  } else {
    uiStore.setActiveAdjustments(props.fileInfo.file_path, {
      brightness: brightness.value,
      contrast: contrast.value,
      saturation: saturation.value,
      hue: hue.value,
      blur: blur.value,
      filter: selectedFilter.value || null,
      rotate: 0,
      flipX: false,
      flipY: false,
      resize: null,
    });
  }

  nextTick(() => {
    scrollSelectedPresetIntoView();
  });
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('AdjustImage');
});

const onImageLoad = () => {
  isProcessing.value = false;
};

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('AdjustImage')) return;

  switch (event.key) {
    case 'ArrowLeft':
      movePresetSelection(-1);
      event.preventDefault();
      event.stopPropagation();
      break;
    case 'ArrowRight':
      movePresetSelection(1);
      event.preventDefault();
      event.stopPropagation();
      break;
    case 'Enter':
      clickSave();
      event.preventDefault();
      event.stopPropagation();
      break;
    case 'Escape':
      clickCancel();
      event.preventDefault();
      event.stopPropagation();
      break;
    default:
      break;
  }
}

const clickCancel = () => {
  emit('cancel');
};

const setEditParams = (overrides: { fileName?: string; destFilePath?: string; outputFormat?: string } = {}) => {
  const name = overrides.fileName || newFileName.value;
  const outputFormat = overrides.outputFormat || fileFormatOptions.value[config.imageEditor.format].label.toLowerCase();

  let destFilePath = overrides.destFilePath;
  if (!destFilePath) {
    destFilePath = getFullPath(getFolderPath(props.fileInfo.file_path), combineFileName(name, outputFormat));
  }

  return {
    sourceFilePath: props.fileInfo.file_path,
    destFilePath,
    outputFormat,
    quality: [90, 80, 60][config.imageEditor.quality] || 80,
    orientation: props.fileInfo.e_orientation || 1,
    flipHorizontal: false,
    flipVertical: false,
    rotate: 0,
    crop: {
      x: 0,
      y: 0,
      width: 0,
      height: 0,
    },
    resize: {
      width: null,
      height: null,
    },
    filter: selectedFilter.value || null,
    brightness: brightness.value !== 0 ? brightness.value : null,
    contrast: contrast.value !== 0 ? contrast.value : null,
    blur: blur.value > 0 ? blur.value : null,
    hue_rotate: hue.value !== 0 ? hue.value : null,
    saturation: saturation.value !== 100 ? saturation.value / 100.0 : null,
  };
};

const executeSave = async (overrides: { fileName?: string; destFilePath?: string; outputFormat?: string } = {}) => {
  isProcessing.value = true;
  let success = false;

  try {
    success = await editImage(setEditParams(overrides));
  } finally {
    isProcessing.value = false;
    if (success) {
      uiStore.updateFileVersion(props.fileInfo.file_path);
      uiStore.clearActiveAdjustments();
      emit('success');
    } else {
      emit('failed');
    }
  }
};

const clickSave = async () => {
  if (isProcessing.value) return;

  if (config.imageEditor.saveAs === 1) {
    isProcessing.value = true;
    try {
      const folderPath = getFolderPath(props.fileInfo.file_path);
      const ext = fileFormatOptions.value[config.imageEditor.format].label.toLowerCase();
      const baseName = newFileName.value;

      let counter = 1;
      let candidateName = `${baseName}_${counter}`;
      let candidatePath = getFullPath(folderPath, combineFileName(candidateName, ext));

      while (await checkFileExists(candidatePath)) {
        counter++;
        candidateName = `${baseName}_${counter}`;
        candidatePath = getFullPath(folderPath, combineFileName(candidateName, ext));
      }

      await executeSave({
        fileName: candidateName,
        destFilePath: candidatePath,
      });
    } catch {
      isProcessing.value = false;
      emit('failed');
    }
  } else {
    showOverwriteConfirm.value = true;
  }
};

</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
