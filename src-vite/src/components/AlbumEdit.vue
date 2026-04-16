<template>
  <ModalDialog :title="isNewAlbum ? $t('album.edit.title_add') : $t('album.edit.title')" @cancel="clickCancel">
    <section class="rounded-box p-3 bg-base-300/30 border border-base-content/5 shadow-sm">
      <!-- two column grid layout -->
      <div class="w-full grid grid-cols-[80px_1fr] gap-x-4 gap-y-2 items-center text-xs">
        <!-- Folder -->
        <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('album.edit.folder') }}</div>
        <div class="h-8 flex items-center justify-between gap-x-2">
          <input v-if="selectedFolder !== ''"
            type="text"
            readonly
            :value="selectedFolder"
            class="w-full bg-transparent border-none p-0 text-xs font-semibold text-base-content/65 focus:border-none focus:ring-0 focus:outline-none"
          />
          <button v-if="selectedFolder === ''"
            class="btn btn-primary btn-sm rounded-box"
            @click="clickSelectFolder"
          >
            <IconNewFolder class="w-4 h-4" />
            {{ $t('album.edit.select_folder') }}
          </button>
          <TButton v-if="isNewAlbum && selectedFolder !== ''"
            :icon="IconNewFolder"
            :selected="true"
            @click="clickSelectFolder"
          />
        </div>

        <!-- Name -->
        <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('album.edit.name') }}</div>
        <div>
          <input
            ref="inputNameRef"
            v-model="inputNameValue"
            type="text"
            maxlength="255"
            :disabled="selectedFolder === ''"
            class="w-full input input-sm text-xs font-semibold"
          />
        </div>

        <!-- Description -->
        <div class="h-8 flex items-start pt-2 text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('album.edit.description') }}</div>
        <div>
          <textarea
            v-model="inputDescriptionValue"
            rows="2"
            maxlength="1024"
            :placeholder="$t('album.edit.description_placeholder')"
            :disabled="selectedFolder === ''"
            class="w-full textarea textarea-sm min-h-[56px] max-h-[200px] text-xs font-semibold"
          ></textarea>
        </div>

        <template v-if="selectedFolder !== ''">
          <!-- Images -->
          <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('album.edit.images') }}</div>
          <div class="h-8 flex items-center text-xs font-semibold text-base-content/65">{{ totalImageCount >= 0 ? $t('album.edit.files_count', {count: totalImageCount.toLocaleString(), size: formatFileSize(totalImageSize) }) : $t('album.edit.files_counting') }}</div>
          <!-- Videos -->
          <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('album.edit.videos') }}</div>
          <div class="h-8 flex items-center text-xs font-semibold text-base-content/65">{{ totalVideoCount >= 0 ? $t('album.edit.files_count', {count: totalVideoCount.toLocaleString(), size: formatFileSize(totalVideoSize) }) : $t('album.info.files_counting') }}</div>
        </template>
        
        <template v-if="!isNewAlbum">
          <!-- Created At -->
          <!-- <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('album.edit.created_at') }}</div>
          <div class="h-8 flex items-center text-xs font-semibold text-base-content/65">{{ createdAt }}</div> -->
          <!-- Modified At -->
          <!-- <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('album.edit.modified_at') }}</div>
          <div class="h-8 flex items-center text-xs font-semibold text-base-content/65">{{ modifiedAt }}</div> -->
          <!-- Last Scan Time -->
          <div class="h-8 flex items-center text-[10px] uppercase tracking-widest font-bold text-base-content/25">{{ $t('album.edit.last_scan_time') }}</div>
          <div class="h-8 flex items-center text-xs font-semibold text-base-content/65">{{ lastScanTime }}</div>
        </template>
      </div>
    </section>

    <!-- cancel and OK buttons -->
    <div class="mt-4 flex justify-end space-x-4">
      <button 
        class="px-4 py-1 rounded-box hover:bg-base-100 hover:text-base-content cursor-pointer" 
        @click="clickCancel"
      >
        {{ $t('msgbox.cancel') }}
      </button>
      <button 
        :class="[
          'px-4 py-1 rounded-box', 
          inputNameValue.trim().length > 0 && !isIndexing ? 'hover:bg-primary hover:text-base-100 cursor-pointer' : 'text-base-content/30 cursor-default',
        ]" 
        @click="clickOk"
      >
        {{ $t('msgbox.ok') }}
      </button>
    </div>
  </ModalDialog>
</template>

<script setup lang="ts">

import { ref, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { countFolder, getAllAlbums } from '@/common/api';
import { useToast } from '@/common/toast';
import { formatFileSize, openFolderDialog, getFolderName } from '@/common/utils';
import { useUIStore } from '@/stores/uiStore';

import ModalDialog from '@/components/ModalDialog.vue';
import TButton from '@/components/TButton.vue';
import { IconNewFolder } from '@/common/icons';

const props = defineProps({
  isNewAlbum: {
    type: Boolean,
    default: false
  },
  albumId: {
    type: Number,
    required: true
  },
  inputName: { 
    type: String, 
    default: '' 
  },
  inputDescription: { 
    type: String, 
    default: '' 
  },
  albumPath: { 
    type: String, 
    default: '' 
  },
  albumCoverFileId: { 
    type: Number, 
    default: null 
  },
  createdAt: { 
    type: String, 
    default: '' 
  },
  modifiedAt: { 
    type: String, 
    default: '' 
  },
  lastScanTime: { 
    type: String, 
    default: '' 
  },
});

const emit = defineEmits(['ok', 'cancel']);
const uiStore = useUIStore();
const { t } = useI18n();

const toast = useToast();

// select folder
const selectedFolder = ref('');

// input 
const inputNameRef = ref<HTMLInputElement | null>(null);
const inputNameValue = ref(props.inputName);
const inputDescriptionValue = ref(props.inputDescription);

// total file count of the album
const totalFolderCount = ref(0);
const totalImageCount = ref(-1);
const totalImageSize = ref(-1);
const totalVideoCount = ref(0);
const totalVideoSize = ref(0);

// image recognition
const isIndexing = ref(false);
const indexedImageCount = ref(0);
// const indexedVideoCount = ref(0);
let shouldStopIndexing = false;

watch(() => selectedFolder.value, (newPath) => {
  if(newPath) {
    if (props.isNewAlbum) {
      // get folder name
      inputNameValue.value = getFolderName(newPath);
      inputDescriptionValue.value = '';
    }

    countFolder(newPath).then((res) => {
      [totalFolderCount.value, totalImageCount.value, totalImageSize.value, totalVideoCount.value, totalVideoSize.value] = res;
      console.log('count folder:', res);
    }).catch((err) => {
      console.error('count folder error:', err);
    });
  }
});

onMounted(async () => {
  window.addEventListener('keydown', handleKeyDown);
  uiStore.pushInputHandler('AlbumEdit');
  
  if (props.isNewAlbum) {
    // clickSelectFolder();
  }
  else {
    selectedFolder.value = props.albumPath;

    setTimeout(() => {
      inputNameRef.value?.focus();
    }, 50); // 50ms delay
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  uiStore.removeInputHandler('AlbumEdit');
  // Stop indexing if component is unmounted
  shouldStopIndexing = true;
});

const clickSelectFolder = async () => {
  const folderPath = await openFolderDialog();
  if (folderPath) {
    selectedFolder.value = folderPath;
  }
};

function handleKeyDown(event: KeyboardEvent) {
  if (!uiStore.isInputActive('AlbumEdit')) return;

  const { key } = event;
  const activeElement = document.activeElement;
  const isInputOrTextarea = activeElement?.tagName === 'INPUT' || activeElement?.tagName === 'TEXTAREA';

  switch (key) {
    case 'Enter':
      // Don't trigger OK if user is in an input or textarea to avoid accidental close, e.g. during IME input
      if (!isInputOrTextarea) {
        event.preventDefault();
        clickOk();
      }
      break;
    case 'Escape':
      clickCancel();
      break;
    default:
      break;
  }
}

const clickOk = async () => {
  if (inputNameValue.value.trim().length > 0 && selectedFolder.value.length > 0) {
    // Check if album with this path already exists
    if (props.isNewAlbum) {
      const albums = await getAllAlbums();
      const exists = albums?.some((album: any) => album.path === selectedFolder.value);
      if (exists) {
        toast.warning(t('tooltip.album_exists'));
        return;
      }
    }
    
    emit(
      'ok', 
      selectedFolder.value,
      inputNameValue.value, 
      inputDescriptionValue.value ? inputDescriptionValue.value : '',
      props.isNewAlbum
    );
  }
};

const clickCancel = () => {
  emit('cancel');
};

</script>
