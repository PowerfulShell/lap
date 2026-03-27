<template>
  <ModalDialog :title="title" :width="440" @cancel="clickCancel">
    <div class="flex flex-col gap-3">
      <div v-if="message" class="text-sm wrap-break-word">
        {{ message }}
      </div>

      <div v-if="filePath" class="flex flex-col gap-1 rounded-box bg-base-100/40 px-3 py-2">
        <div class="text-xs text-base-content/50">{{ fileLabel }}</div>
        <div class="text-sm text-base-content font-medium break-all">{{ fileName }}</div>
        <div class="text-xs text-base-content/60 break-all">{{ filePath }}</div>
      </div>

      <label class="mt-1 flex items-center gap-2 text-sm cursor-pointer">
        <input v-model="skipThisFile" type="checkbox" class="checkbox checkbox-primary checkbox-sm" />
        <span>{{ skipLabel }}</span>
      </label>

      <div class="mt-2 flex justify-end space-x-4">
        <button
          class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer"
          @click="clickCancel"
        >
          {{ cancelText }}
        </button>

        <button
          class="px-4 py-1 rounded-box hover:bg-primary hover:text-primary-content cursor-pointer"
          @click="clickContinue"
        >
          {{ continueText }}
        </button>
      </div>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useUIStore } from '@/stores/uiStore';
import ModalDialog from '@/components/ModalDialog.vue';

const props = defineProps({
  title: {
    type: String,
    required: true,
  },
  message: {
    type: String,
    default: '',
  },
  fileLabel: {
    type: String,
    default: '',
  },
  filePath: {
    type: String,
    default: '',
  },
  continueText: {
    type: String,
    default: 'Continue',
  },
  skipLabel: {
    type: String,
    default: 'Skip this file and continue',
  },
  cancelText: {
    type: String,
    default: 'Cancel',
  },
});

const emit = defineEmits(['continue', 'cancel']);
const uiStore = useUIStore();
const skipThisFile = ref(false);
const fileName = computed(() => {
  if (!props.filePath) return '';
  const normalized = props.filePath.replace(/\\/g, '/');
  return normalized.split('/').pop() || props.filePath;
});

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('IndexRecoveryDialog')) return;

  switch (event.key) {
    case 'Enter':
      clickContinue();
      break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

function clickContinue() {
  emit('continue', skipThisFile.value);
}

function clickCancel() {
  emit('cancel');
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('IndexRecoveryDialog');
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('IndexRecoveryDialog');
});
</script>
