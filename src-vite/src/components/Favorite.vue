<template>
  <div class="sidebar-panel">
    <div class="sidebar-panel-header">
      <div role="tablist" class="sidebar-header-tabs">
        <button
          role="tab"
          :class="['sidebar-header-tab', activeTab === 'favorite' ? 'tab-active' : '']"
          @click="setActiveTab('favorite')"
        >
          {{ $t('favorite.files') }}
        </button>
        <button
          role="tab"
          :class="['sidebar-header-tab', activeTab === 'rating' ? 'tab-active' : '']"
          @click="setActiveTab('rating')"
        >
          {{ $t('favorite.ratings') }}
        </button>
      </div>
    </div>

    <div v-if="activeTab === 'favorite'" class="flex-1 overflow-x-hidden overflow-y-auto">
      <div
        :class="[
          'sidebar-item',
          libConfig.favorite.folderId === 0 && libConfig.favorite.rating === null ? 'sidebar-item-selected' : 'sidebar-item-hover',
        ]"
        @click="clickFavoriteFiles()"
      >
        <IconHeart class="mx-1 w-5 h-5 shrink-0" />
        <div class="sidebar-item-label">
          {{ $t('favorite.files') }}
        </div>
        <span v-if="favoriteFilesCount" class="sidebar-item-count">{{ favoriteFilesCount.toLocaleString() }}</span>
      </div>

      <div v-if="favorite_folders.length > 0" class="sidebar-panel-header">
        <span class="sidebar-panel-header-title">{{ $t('favorite.folders') }}</span>
      </div>
      <div class="grow overflow-x-hidden overflow-y-auto">
        <ul>
          <li v-for="folder in favorite_folders" :key="folder.id">
            <div
              :class="[
                'sidebar-item group',
                libConfig.favorite.folderId === folder.id ? 'sidebar-item-selected' : 'sidebar-item-hover',
              ]"
              @click="clickFavoriteFolder(folder)"
            >
              <IconFolderFavorite class="mx-1 h-5 shrink-0" />
              <div class="sidebar-item-label">
                {{ folder.name }}
              </div>
              <ContextMenu
                :class="[
                  'ml-auto flex flex-row items-center text-base-content/30',
                  libConfig.favorite.folderId != folder.id ? 'invisible group-hover:visible' : ''
                ]"
                :iconMenu="IconMore"
                :menuItems="favoriteFolderMenuItems"
                :smallIcon="true"
              />
            </div>
          </li>
        </ul>
      </div>
    </div>

    <div v-else class="flex-1 overflow-x-hidden overflow-y-auto">
      <ul>
        <li>
          <div
            :class="[
              'sidebar-item',
              libConfig.favorite.rating === 0 ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="clickRating(0)"
          >
            <div class="mx-1 flex items-center gap-2">
              <IconStar class="w-5 h-5 shrink-0" />
              <span>{{ $t('favorite.unrated') }}</span>
            </div>
            <span v-if="unratedCount" class="sidebar-item-count ml-auto">{{ unratedCount.toLocaleString() }}</span>
          </div>
        </li>
        <li v-for="rating in [5, 4, 3, 2, 1]" :key="rating">
          <div
            :class="[
              'sidebar-item',
              libConfig.favorite.rating === rating ? 'sidebar-item-selected' : 'sidebar-item-hover',
            ]"
            @click="clickRating(rating)"
          >
            <div class="mx-1 flex items-center gap-0.5">
              <IconStarFilled
                v-for="index in rating"
                :key="index"
                class="w-5 h-5 shrink-0"
              />
            </div>
            <span v-if="ratingCounts[rating]" class="sidebar-item-count ml-auto">{{ ratingCounts[rating].toLocaleString() }}</span>
          </div>
        </li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { libConfig } from '@/common/config';

import { useI18n } from 'vue-i18n';
import { getQueryCountAndSum, getFavoriteFolders, setFolderFavorite } from '@/common/api';
import { getFolderName } from '@/common/utils';
import ContextMenu from '@/components/ContextMenu.vue';
import { IconMore, IconFolderFavorite, IconHeart, IconStar, IconStarFilled } from '@/common/icons';

const props = defineProps({
  titlebar: {
    type: String,
    required: true
  }
});

const activeTab = computed(() => {
  return (libConfig.favorite as any).tab === 'rating' ? 'rating' : 'favorite';
});

const favoriteFilesCount = ref(0);
const unratedCount = ref(0);
const ratingCounts = ref<Record<number, number>>({
  1: 0,
  2: 0,
  3: 0,
  4: 0,
  5: 0,
});

const buildQueryParams = ({ isFavorite = false, rating = -1 } = {}) => ({
  searchFileType: 0,
  sortType: 0,
  sortOrder: 0,
  searchFileName: "",
  searchAllSubfolders: "",
  searchFolder: "",
  startDate: 0,
  endDate: 0,
  make: "",
  model: "",
  lensMake: "",
  lensModel: "",
  locationAdmin1: "",
  locationName: "",
  isFavorite,
  rating,
  tagId: 0,
  personId: 0,
});

async function loadCounts() {
  const total = await getQueryCountAndSum(buildQueryParams({ isFavorite: true }));
  favoriteFilesCount.value = total ? Number(total[0]) : 0;

  const unrated = await getQueryCountAndSum(buildQueryParams({ rating: 0 }));
  unratedCount.value = unrated ? Number(unrated[0]) : 0;

  const entries = await Promise.all(
    [1, 2, 3, 4, 5].map(async (rating) => {
      const result = await getQueryCountAndSum(buildQueryParams({ rating }));
      return [rating, result ? Number(result[0]) : 0] as const;
    }),
  );

  ratingCounts.value = Object.fromEntries(entries) as Record<number, number>;
}

onMounted(() => {
  if ((libConfig.favorite as any).tab !== 'favorite' && (libConfig.favorite as any).tab !== 'rating') {
    (libConfig.favorite as any).tab = 'favorite';
  }
  void loadCounts();
});

function setActiveTab(tab: 'favorite' | 'rating') {
  if (tab === 'favorite') {
    clickFavoriteFiles();
    return;
  }

  if (libConfig.favorite.rating === null) {
    clickRating(0);
    return;
  }

  (libConfig.favorite as any).tab = 'rating';
}

const { locale, messages } = useI18n();
const localeMsg = computed(() => messages.value[locale.value] as any);
interface FavoriteFolder {
  id: number;
  album_id: number;
  path: string;
  name?: string;
}
const favorite_folders = ref<FavoriteFolder[]>([]);
const favoriteFolderMenuItems = computed(() => {
  return [
    {
      label: localeMsg.value.menu.meta.unfavorite,
      icon: IconHeart,
      action: () => {
        UnFavorite();
      }
    },
  ];
});
onMounted(() => {
  if (favorite_folders.value.length === 0) {
    getFavoriteFolders().then((folders) => {
      favorite_folders.value = folders || [];
      favorite_folders.value.forEach((folder) => {
        folder.name = getFolderName(folder.path);
      });
    });
  }
});
// click favorite files
function clickFavoriteFiles() {
  (libConfig.favorite as any).tab = 'favorite';
  libConfig.favorite.albumId = null;
  libConfig.favorite.folderId = 0;
  libConfig.favorite.folderPath = '';
  libConfig.favorite.rating = null;
}

if (libConfig.favorite.folderId !== 0) {
  clickFavoriteFiles();
}

function clickRating(rating: number) {
  (libConfig.favorite as any).tab = 'rating';
  libConfig.favorite.albumId = null;
  libConfig.favorite.folderId = 0;
  libConfig.favorite.folderPath = '';
  libConfig.favorite.rating = rating;
}

if (libConfig.favorite.folderId !== 0 || libConfig.favorite.rating !== null) {
  (libConfig.favorite as any).tab = 'rating';
} else {
  clickFavoriteFiles();
}

function clickFavoriteFolder(folder: any) {
  libConfig.favorite.albumId = folder.album_id;
  libConfig.favorite.folderId = folder.id;
  libConfig.favorite.folderPath = folder.path;
}
function UnFavorite() {
  setFolderFavorite(libConfig.favorite.folderId, false).then(() => {
    const index = favorite_folders.value.findIndex((f: any) => f.id === libConfig.favorite.folderId);
    favorite_folders.value = favorite_folders.value.filter((f: any) => f.id !== libConfig.favorite.folderId);
    if (favorite_folders.value.length === 0) {
      libConfig.favorite.folderId = 0;
      libConfig.favorite.albumId = null;
      libConfig.favorite.folderPath = '';
    } else if (index === 0) {
      libConfig.favorite.folderId = favorite_folders.value[index].id;
      libConfig.favorite.albumId = favorite_folders.value[index].album_id;
      libConfig.favorite.folderPath = favorite_folders.value[index].path;
    } else {
      libConfig.favorite.folderId = favorite_folders.value[index - 1].id;
      libConfig.favorite.albumId = favorite_folders.value[index - 1].album_id;
      libConfig.favorite.folderPath = favorite_folders.value[index - 1].path;
    }
  });
}
</script>
