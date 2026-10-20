<script setup>
import { computed, nextTick, onMounted, onBeforeUnmount, ref, watch } from 'vue'
import { ICONS } from '../icons.js'

const props = defineProps({
  isOpen: {
    type: Boolean,
    default: false,
  },
  rootPath: {
    type: String,
    default: '~',
  },
  theme: {
    type: String,
    default: 'dark',
  },
})

const emit = defineEmits(['close', 'open-terminal', 'set-session-cwd'])

const dockRef = ref(null)
const listContainerRef = ref(null)
const searchInputRef = ref(null)
const rowRefs = ref([])

const currentPath = ref('~')
const parentPath = ref(null)
const entries = ref([])
const loading = ref(false)
const error = ref('')
const filterQuery = ref('')
const showHidden = ref(false)
const isEditingPath = ref(false)
const pathInput = ref('~')
const copiedNotice = ref('')
const previewPath = ref(null)
const previewContent = ref('')
const previewLoading = ref(false)

const selectedIndex = ref(0)
const isDockFocused = ref(false)
const isDockHovered = ref(false)

function formatBytes(bytes) {
  if (bytes === 0 || !bytes) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}

function getFileIcon(entry) {
  if (entry.is_dir) return 'folder'
  const ext = entry.ext?.toLowerCase() || ''
  if (['js', 'ts', 'jsx', 'tsx', 'vue', 'py', 'rs', 'go', 'c', 'cpp', 'java', 'html', 'css'].includes(ext)) {
    return 'code'
  }
  if (['json', 'yaml', 'yml', 'toml', 'xml', 'md', 'txt', 'log'].includes(ext)) {
    return 'doc'
  }
  return 'file'
}

const filteredEntries = computed(() => {
  let list = entries.value
  if (!showHidden.value) {
    list = list.filter((e) => !e.name.startsWith('.'))
  }
  const q = filterQuery.value.trim().toLowerCase()
  if (q) {
    list = list.filter((e) => e.name.toLowerCase().includes(q))
  }
  return list
})

function scrollToSelected() {
  nextTick(() => {
    const el = rowRefs.value[selectedIndex.value]
    if (el) {
      el.scrollIntoView({ block: 'nearest' })
    }
  })
}

function moveSelection(delta) {
  if (!filteredEntries.value.length) return
  const maxIdx = filteredEntries.value.length - 1
  selectedIndex.value = Math.max(0, Math.min(maxIdx, selectedIndex.value + delta))
  scrollToSelected()
}

async function loadDir(target, highlightName = null) {
  loading.value = true
  error.value = ''
  try {
    const res = await window.vibedeck.readDir(target || currentPath.value)
    currentPath.value = res.current_path
    pathInput.value = res.current_path
    parentPath.value = res.parent_path
    entries.value = res.entries || []

    await nextTick()
    if (highlightName) {
      const idx = filteredEntries.value.findIndex((e) => e.name === highlightName)
      selectedIndex.value = idx >= 0 ? idx : 0
    } else {
      selectedIndex.value = 0
    }
    scrollToSelected()
  } catch (err) {
    error.value = String(err)
  } finally {
    loading.value = false
  }
}

function handleGoUp() {
  if (parentPath.value) {
    const leavingDirName = currentPath.value.split('/').filter(Boolean).pop()
    loadDir(parentPath.value, leavingDirName)
  }
}

function handleEnterEntry(entry, index) {
  if (typeof index === 'number') {
    selectedIndex.value = index
  }
  if (entry.is_dir) {
    loadDir(entry.path)
  } else {
    handlePreviewFile(entry)
  }
}

function activateCurrentItem() {
  const entry = filteredEntries.value[selectedIndex.value]
  if (!entry) return
  if (entry.is_dir) {
    loadDir(entry.path)
  } else {
    handlePreviewFile(entry)
  }
}

function navigateToInput() {
  isEditingPath.value = false
  if (pathInput.value.trim()) {
    loadDir(pathInput.value.trim())
  }
}

async function copyPath(path) {
  try {
    await navigator.clipboard.writeText(path)
    copiedNotice.value = 'Path tersalin ke clipboard!'
    setTimeout(() => {
      copiedNotice.value = ''
    }, 2000)
  } catch (e) {
    console.error('Clipboard copy failed', e)
  }
}

function spawnTerminalIn(path) {
  emit('open-terminal', path)
  copiedNotice.value = `Terminal dibuka di: ${path.split('/').pop() || path}`
  setTimeout(() => {
    copiedNotice.value = ''
  }, 2500)
}

function setAsSessionCwd(path) {
  emit('set-session-cwd', path)
  copiedNotice.value = `Workspace path diatur ke: ${path}`
  setTimeout(() => {
    copiedNotice.value = ''
  }, 2500)
}

async function handlePreviewFile(entry) {
  previewPath.value = entry.path
  previewLoading.value = true
  previewContent.value = ''
  try {
    const text = await window.vibedeck.readFilePreview(entry.path)
    previewContent.value = text || '(File kosong)'
  } catch (err) {
    previewContent.value = `Gagal memuat preview: ${err}`
  } finally {
    previewLoading.value = false
  }
}

function closePreview() {
  previewPath.value = null
  previewContent.value = ''
}

async function revealFinder(path) {
  if (window.vibedeck.revealInFinder) {
    await window.vibedeck.revealInFinder(path)
  }
}

function handleKeydown(e) {
  if (!props.isOpen) return

  // Check if focus is inside search input or path input
  const activeEl = document.activeElement
  const isInsideDock = dockRef.value && (dockRef.value === activeEl || dockRef.value.contains(activeEl) || isDockHovered.value)

  if (!isInsideDock) return

  const isSearchInput = activeEl === searchInputRef.value
  const isPathInput = isEditingPath.value

  if (isPathInput) {
    if (e.key === 'Escape') {
      isEditingPath.value = false
      dockRef.value?.focus()
    }
    return
  }

  if (isSearchInput) {
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      moveSelection(1)
      dockRef.value?.focus()
      return
    }
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      moveSelection(-1)
      dockRef.value?.focus()
      return
    }
    if (e.key === 'Enter') {
      e.preventDefault()
      activateCurrentItem()
      return
    }
    if (e.key === 'Escape') {
      e.preventDefault()
      if (filterQuery.value) {
        filterQuery.value = ''
      } else {
        dockRef.value?.focus()
      }
      return
    }
    return // normal typing
  }

  // File list navigation
  if (e.key === 'ArrowDown' || e.key === 'j') {
    e.preventDefault()
    moveSelection(1)
  } else if (e.key === 'ArrowUp' || e.key === 'k') {
    e.preventDefault()
    moveSelection(-1)
  } else if (e.key === 'ArrowRight' || e.key === 'l' || e.key === 'Enter') {
    e.preventDefault()
    activateCurrentItem()
  } else if (e.key === 'ArrowLeft' || e.key === 'h' || e.key === 'Backspace') {
    e.preventDefault()
    if (previewPath.value) {
      closePreview()
    } else {
      handleGoUp()
    }
  } else if (e.key === ' ') {
    e.preventDefault()
    const entry = filteredEntries.value[selectedIndex.value]
    if (entry) {
      if (entry.is_dir) {
        loadDir(entry.path)
      } else {
        if (previewPath.value === entry.path) {
          closePreview()
        } else {
          handlePreviewFile(entry)
        }
      }
    }
  } else if (e.key === 'Home') {
    e.preventDefault()
    selectedIndex.value = 0
    scrollToSelected()
  } else if (e.key === 'End') {
    e.preventDefault()
    selectedIndex.value = Math.max(0, filteredEntries.value.length - 1)
    scrollToSelected()
  } else if (e.key === 'PageDown') {
    e.preventDefault()
    moveSelection(10)
  } else if (e.key === 'PageUp') {
    e.preventDefault()
    moveSelection(-10)
  } else if (e.key.toLowerCase() === 't' && !e.metaKey && !e.ctrlKey) {
    e.preventDefault()
    const entry = filteredEntries.value[selectedIndex.value]
    if (entry) {
      spawnTerminalIn(entry.is_dir ? entry.path : currentPath.value)
    }
  } else if (e.key.toLowerCase() === 'c' && !e.metaKey && !e.ctrlKey) {
    e.preventDefault()
    const entry = filteredEntries.value[selectedIndex.value]
    if (entry) {
      copyPath(entry.path)
    }
  } else if (e.key.toLowerCase() === 'o' && !e.metaKey && !e.ctrlKey) {
    e.preventDefault()
    const entry = filteredEntries.value[selectedIndex.value]
    if (entry) {
      revealFinder(entry.path)
    }
  } else if (e.key === '/' && !e.metaKey && !e.ctrlKey) {
    e.preventDefault()
    searchInputRef.value?.focus()
    searchInputRef.value?.select()
  } else if (e.key === 'Escape') {
    e.preventDefault()
    if (previewPath.value) {
      closePreview()
    } else if (filterQuery.value) {
      filterQuery.value = ''
    } else {
      emit('close')
    }
  }
}

watch(() => filteredEntries.value.length, (len) => {
  if (selectedIndex.value >= len) {
    selectedIndex.value = Math.max(0, len - 1)
  }
  rowRefs.value = []
})

watch(() => props.rootPath, (newVal) => {
  if (newVal) {
    loadDir(newVal)
  }
})

watch(() => props.isOpen, (open) => {
  if (open) {
    isDockFocused.value = true
    if (entries.value.length === 0) {
      loadDir(props.rootPath || '~')
    }
    nextTick(() => {
      dockRef.value?.focus()
    })
  } else {
    isDockFocused.value = false
  }
})

onMounted(() => {
  loadDir(props.rootPath || '~')
  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <aside
    ref="dockRef"
    class="file-manager-dock"
    :class="{ open: isOpen, focused: isDockFocused }"
    tabindex="0"
    @focus="isDockFocused = true"
    @blur="isDockFocused = false"
    @mouseenter="isDockHovered = true"
    @mouseleave="isDockHovered = false"
    @click="dockRef?.focus()"
  >
    <div class="fm-header">
      <div class="fm-title-group">
        <svg class="fm-title-icon" viewBox="0 0 24 24" v-html="ICONS.files"></svg>
        <span class="fm-title">VIBE FILES</span>
        <span class="fm-badge">KITTY-LITE</span>
      </div>
      <div class="fm-header-actions">
        <button
          class="fm-icon-btn"
          :class="{ active: showHidden }"
          :title="showHidden ? 'Sembunyikan Dotfiles (.)' : 'Tampilkan Dotfiles (.)'"
          @click.stop="showHidden = !showHidden"
        >
          <span style="font-size: 11px; font-weight: 700;">.*</span>
        </button>
        <button class="fm-icon-btn" title="Refresh Direktori" @click.stop="loadDir(currentPath)">
          <svg viewBox="0 0 24 24" v-html="ICONS.restart"></svg>
        </button>
        <button class="fm-icon-btn close" title="Tutup File Manager (⌘E)" @click.stop="emit('close')">
          <svg viewBox="0 0 24 24" v-html="ICONS.x"></svg>
        </button>
      </div>
    </div>

    <!-- Quick Navigation Presets -->
    <div class="fm-quick-bar">
      <button class="fm-quick-btn" title="Home (~)" @click.stop="loadDir('~')">
        <svg viewBox="0 0 24 24" v-html="ICONS.home"></svg>
        <span>~</span>
      </button>
      <button class="fm-quick-btn" title="Projects (~/Documents/projects)" @click.stop="loadDir('~/Documents/projects')">
        <span>projects</span>
      </button>
      <button class="fm-quick-btn" title="Workspace Root Sesi" @click.stop="loadDir(rootPath)">
        <span>ws-root</span>
      </button>
      <span class="spacer"></span>
      <button v-if="parentPath" class="fm-quick-btn up" title="Naik ke Folder Induk (Arrow Left / Backspace)" @click.stop="handleGoUp">
        <svg viewBox="0 0 24 24" v-html="ICONS.arrowUp"></svg>
        <span>Up</span>
      </button>
    </div>

    <!-- Current Path Breadcrumb & Editable Input -->
    <div class="fm-path-bar">
      <div v-if="!isEditingPath" class="fm-breadcrumb" :title="`Klik untuk ketik path langsung: ${currentPath}`" @click.stop="isEditingPath = true">
        <svg class="fm-crumb-icon" viewBox="0 0 24 24" v-html="ICONS.folder"></svg>
        <span class="fm-crumb-text">{{ currentPath }}</span>
        <span class="fm-crumb-edit">
          <svg viewBox="0 0 24 24" v-html="ICONS.edit"></svg>
        </span>
      </div>
      <input
        v-else
        v-model="pathInput"
        class="fm-path-input"
        autofocus
        @keydown.enter="navigateToInput"
        @keydown.esc="isEditingPath = false"
        @blur="navigateToInput"
        @click.stop
      />
    </div>

    <!-- Search / Filter Input -->
    <div class="fm-search-bar" @click.stop>
      <svg class="fm-search-icon" viewBox="0 0 24 24" v-html="ICONS.search"></svg>
      <input
        ref="searchInputRef"
        v-model="filterQuery"
        class="fm-search-input"
        placeholder="Cari file (Tekan / untuk fokus)..."
      />
      <button v-if="filterQuery" class="fm-clear-btn" @click.stop="filterQuery = ''">
        <svg viewBox="0 0 24 24" v-html="ICONS.x"></svg>
      </button>
    </div>

    <!-- Navigation Help Banner -->
    <div class="fm-hints-bar">
      <span><kbd>↑↓</kbd> Navigasi</span>
      <span><kbd>→</kbd> Buka</span>
      <span><kbd>←</kbd> Induk</span>
      <span><kbd>Space</kbd> Preview</span>
      <span><kbd>T</kbd> Terminal</span>
    </div>

    <!-- Toast Notification for Copied Path / Action -->
    <div v-if="copiedNotice" class="fm-toast">
      <svg viewBox="0 0 24 24" v-html="ICONS.check"></svg>
      <span>{{ copiedNotice }}</span>
    </div>

    <!-- Error Notice -->
    <div v-if="error" class="fm-error-box">
      <span>⚠️ {{ error }}</span>
      <button class="fm-mini-btn" @click.stop="loadDir('~')">Kembali ke ~</button>
    </div>

    <!-- File and Directory List -->
    <div
      ref="listContainerRef"
      class="fm-list-container"
      tabindex="-1"
    >
      <div v-if="loading" class="fm-loading">
        <span class="fm-spinner"></span>
        <span>Membaca filesystem...</span>
      </div>

      <div v-else-if="filteredEntries.length === 0" class="fm-empty">
        <svg viewBox="0 0 24 24" v-html="ICONS.folder"></svg>
        <span>Tidak ada file atau folder ditemukan.</span>
      </div>

      <div v-else class="fm-entries-list">
        <div
          v-for="(entry, index) in filteredEntries"
          :key="entry.path"
          :ref="(el) => { if (el) rowRefs[index] = el }"
          class="fm-entry-row"
          :class="{
            'is-dir': entry.is_dir,
            'selected': selectedIndex === index,
          }"
          @click.stop="handleEnterEntry(entry, index)"
        >
          <div class="fm-entry-icon" :class="entry.is_dir ? 'dir' : 'file'">
            <svg viewBox="0 0 24 24" v-html="ICONS[getFileIcon(entry)]"></svg>
          </div>

          <div class="fm-entry-info">
            <span class="fm-entry-name" :title="entry.name">{{ entry.name }}</span>
            <span v-if="!entry.is_dir" class="fm-entry-meta">{{ formatBytes(entry.size) }}</span>
            <span v-else class="fm-entry-meta dir-meta">DIR</span>
          </div>

          <!-- Quick Action Buttons -->
          <div class="fm-entry-actions" @click.stop>
            <!-- Open Terminal Here -->
            <button
              class="fm-act-btn"
              :title="entry.is_dir ? 'Buka Terminal Baru di folder ini (T)' : 'Buka Terminal Baru di folder induk (T)'"
              @click.stop="spawnTerminalIn(entry.is_dir ? entry.path : currentPath)"
            >
              <svg viewBox="0 0 24 24" v-html="ICONS.terminal"></svg>
            </button>

            <!-- Preview File -->
            <button
              v-if="!entry.is_dir"
              class="fm-act-btn"
              title="Lihat Preview File (Space)"
              @click.stop="handlePreviewFile(entry)"
            >
              <svg viewBox="0 0 24 24" v-html="ICONS.eye"></svg>
            </button>

            <!-- Set as Session Root -->
            <button
              v-if="entry.is_dir"
              class="fm-act-btn"
              title="Jadikan Root Path Workspace Sesi"
              @click.stop="setAsSessionCwd(entry.path)"
            >
              <svg viewBox="0 0 24 24" v-html="ICONS.folder"></svg>
            </button>

            <!-- Copy Full Path -->
            <button
              class="fm-act-btn"
              title="Salin Full Path (C)"
              @click.stop="copyPath(entry.path)"
            >
              <svg viewBox="0 0 24 24" v-html="ICONS.copy"></svg>
            </button>

            <!-- Reveal in Finder on Mac -->
            <button
              class="fm-act-btn"
              title="Buka di Finder (O)"
              @click.stop="revealFinder(entry.path)"
            >
              <svg viewBox="0 0 24 24" v-html="ICONS.external"></svg>
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Quick File Preview Overlay / Drawer -->
    <div v-if="previewPath" class="fm-preview-drawer" @click.stop>
      <div class="fm-preview-header">
        <div class="fm-preview-title" :title="previewPath">
          <svg viewBox="0 0 24 24" v-html="ICONS.doc"></svg>
          <span>{{ previewPath.split('/').pop() }}</span>
        </div>
        <div class="fm-preview-actions">
          <button class="fm-icon-btn" title="Salin Path (C)" @click="copyPath(previewPath)">
            <svg viewBox="0 0 24 24" v-html="ICONS.copy"></svg>
          </button>
          <button class="fm-icon-btn close" title="Tutup Preview (Esc / Arrow Left)" @click="closePreview">
            <svg viewBox="0 0 24 24" v-html="ICONS.x"></svg>
          </button>
        </div>
      </div>
      <div class="fm-preview-body">
        <div v-if="previewLoading" class="fm-loading">Memuat konten...</div>
        <pre v-else class="fm-preview-text">{{ previewContent }}</pre>
      </div>
    </div>
  </aside>
</template>
