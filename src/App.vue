<script setup>
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue'
import TerminalPanel from './components/TerminalPanel.vue'
import FileManager from './components/FileManager.vue'
import { AGENTS } from './agents.js'
import { ICONS } from './icons.js'

const branch = ref(null)
const lastFail = ref(null)
const clock = ref('')
const dateStr = ref('')

const THEMES = [
  { id: 'dark', label: 'Dark', icon: 'moon', desc: 'Cyberpunk Neon Cyan' },
  { id: 'light', label: 'Light', icon: 'sun', desc: 'Clean Modern White' },
  { id: 'midnight', label: 'Midnight', icon: 'sparkles', desc: 'Deep Violet OLED' },
  { id: 'matrix', label: 'Matrix', icon: 'terminal', desc: 'Hacker Phosphor Green' },
  { id: 'nord', label: 'Nord', icon: 'cloud', desc: 'Arctic Frost Blue' },
]
const currentTheme = ref(localStorage.getItem('vibedeck-theme') || 'dark')

function setTheme(t) {
  currentTheme.value = t
  localStorage.setItem('vibedeck-theme', t)
  document.documentElement.setAttribute('data-theme', t)
  logActivity(`Theme changed to ${t.toUpperCase()}`, 'info')
}

function cycleTheme() {
  const i = THEMES.findIndex((x) => x.id === currentTheme.value)
  const next = THEMES[(i + 1) % THEMES.length].id
  setTheme(next)
}

const sessions = ref([
  { id: 1, name: 'Workspace', cwd: '~', panels: [], counters: {} },
])
const activeId = ref(1)
const view = ref('terminal') // 'terminal', 'agents', 'logs', 'settings'
const expanded = ref(false)
const layoutMode = ref('grid') // 'grid', 'columns', 'stack'
const maximizedPanelId = ref(null)
const showFileManager = ref(false)
const showPalette = ref(false)
const showShortcuts = ref(false)
const showPathModal = ref(false)
const customPathInput = ref('~')
const pathError = ref('')
const paletteSearch = ref('')
const editingSessionId = ref(null)
const editSessionName = ref('')

const PRESET_PATHS = [
  '~',
  '~/Documents/projects',
  '~/Code',
  '~/Projects',
  '~/Desktop',
  '/',
]

const MENU = [
  { id: 'terminal', icon: 'terminal', label: 'Terminal Grid', view: 'terminal' },
  { id: 'files', icon: 'files', label: 'File Manager (⌘E)', action: 'files' },
  { id: 'agents', icon: 'bot', label: 'Agent Hub', view: 'agents' },
  { id: 'logs', icon: 'doc', label: 'Activity Logs', view: 'logs' },
  { id: 'search', icon: 'search', label: 'Command (⌘K)', action: 'palette' },
]

const FOOT = [
  { id: 'shortcuts', icon: 'keyboard', label: 'Shortcuts (?)', action: 'shortcuts' },
  { id: 'settings', icon: 'gear', label: 'Settings', view: 'settings' },
]

let uid = 0
let sid = 1
const active = computed(() => sessions.value.find((x) => x.id === activeId.value) || sessions.value[0])
const allPanels = computed(() => sessions.value.flatMap((x) => x.panels))
let timer

const total = computed(() => allPanels.value.length)
const runningAgents = computed(() => allPanels.value.filter((p) => p.kind !== 'terminal' && p.alive !== false).length)
const aiUsed = computed(() => {
  const kinds = [...new Set(allPanels.value.map((p) => p.kind).filter((k) => k !== 'terminal'))]
  return kinds.map((k) => AGENTS[k]?.label || k).join(', ') || 'None'
})

const activityLogs = ref([])

function logActivity(text, type = 'info') {
  const time = new Date().toLocaleTimeString('id-ID', { hour12: false }).replace(/\./g, ':')
  activityLogs.value.unshift({ id: Date.now() + Math.random(), time, text, type })
  if (activityLogs.value.length > 50) activityLogs.value.pop()
}

function openPathEditor() {
  customPathInput.value = active.value?.cwd || '~'
  pathError.value = ''
  showPathModal.value = true
}

async function saveWorkspacePath() {
  const target = customPathInput.value.trim() || '~'
  const bridge = window.vibedeck
  const exists = await bridge.verifyPath(target)
  if (!exists && target !== '~' && target !== '/') {
    pathError.value = 'Direktori tidak ditemukan di sistem'
    return
  }
  if (active.value) {
    active.value.cwd = target
    logActivity(`Workspace directory updated to: ${target}`, 'info')
    refreshGit()
  }
  showPathModal.value = false
}

function selectPresetPath(p) {
  customPathInput.value = p
  pathError.value = ''
}

function add(kind, customCwd = null) {
  if (!active.value) return
  view.value = 'terminal'
  const c = active.value.counters
  c[kind] = (c[kind] || 0) + 1
  const panelId = `p${++uid}`
  const targetCwd = customCwd || active.value.cwd || '~'
  active.value.panels.push({
    id: panelId,
    kind,
    n: c[kind],
    alive: true,
    width: null,
    height: null,
    cwd: targetCwd,
  })
  logActivity(`Spawned ${AGENTS[kind]?.label || kind} in ${targetCwd}`, 'spawn')
}

function handleFmOpenTerminal(path) {
  add('terminal', path)
}

function handleFmSetSessionCwd(path) {
  if (active.value) {
    active.value.cwd = path
    refreshGit()
    logActivity(`Workspace directory updated from File Manager: ${path}`, 'info')
  }
}

function addSession(path) {
  const newCwd = path || '~'
  sessions.value.push({ id: ++sid, name: `Session ${sid}`, cwd: newCwd, panels: [], counters: {} })
  activeId.value = sid
  view.value = 'terminal'
  add('terminal')
  logActivity(`Created session: Session ${sid} (Path: ${newCwd})`, 'info')
}

function closeSession(id) {
  if (sessions.value.length <= 1) return
  const i = sessions.value.findIndex((x) => x.id === id)
  const target = sessions.value[i]
  sessions.value.splice(i, 1)
  if (activeId.value === id) {
    activeId.value = sessions.value[Math.max(0, i - 1)].id
  }
  logActivity(`Closed session: ${target?.name}`, 'warn')
}

function startRename(s) {
  editingSessionId.value = s.id
  editSessionName.value = s.name
}

function saveRename(s) {
  if (editSessionName.value.trim()) {
    s.name = editSessionName.value.trim()
  }
  editingSessionId.value = null
}

let dragId = null
const overId = ref(null)
const dropPos = ref('after')

function dragStart(id, e) {
  dragId = id
  e.dataTransfer.effectAllowed = 'move'
  e.dataTransfer.setData('text/plain', id)
}

function dragEnd() {
  dragId = null
  overId.value = null
  nextTick(() => {
    window.dispatchEvent(new Event('resize'))
  })
}

function onPanelDragOver(id, e) {
  if (id === dragId) return
  overId.value = id
  const rect = e.currentTarget.getBoundingClientRect()
  dropPos.value = (e.clientX - rect.left) < (rect.width / 2) ? 'before' : 'after'
}

function onPanelDrop(id) {
  if (!dragId || dragId === id) {
    overId.value = null
    dragId = null
    return
  }
  const list = active.value.panels
  const from = list.findIndex((p) => p.id === dragId)
  if (from < 0) return
  const [m] = list.splice(from, 1)
  let to = list.findIndex((p) => p.id === id)
  if (to < 0) {
    list.push(m)
  } else if (dropPos.value === 'after') {
    list.splice(to + 1, 0, m)
  } else {
    list.splice(to, 0, m)
  }
  overId.value = null
  dragId = null
  logActivity(`Reordered terminal window`, 'info')
  nextTick(() => {
    window.dispatchEvent(new Event('resize'))
  })
}

function handlePanelResize({ id, width, height }) {
  const p = active.value?.panels.find((x) => x.id === id)
  if (p) {
    if (width !== undefined) p.width = width
    if (height !== undefined) p.height = height
  }
}

function handlePanelRename(id, title) {
  const p = active.value?.panels.find((x) => x.id === id)
  if (p) {
    p.title = title
    logActivity(`Renamed terminal window to: ${title}`, 'info')
  }
}

function jump(sessionId) {
  activeId.value = sessionId
  view.value = 'terminal'
}

function close(id) {
  if (!active.value) return
  const target = active.value.panels.find((p) => p.id === id)
  active.value.panels = active.value.panels.filter((p) => p.id !== id)
  if (maximizedPanelId.value === id) maximizedPanelId.value = null
  if (target) {
    logActivity(`Closed panel ${AGENTS[target.kind]?.label || target.kind}`, 'info')
  }
}

function toggleMaximize(id) {
  maximizedPanelId.value = maximizedPanelId.value === id ? null : id
}

function onExit(panel, code) {
  if (code !== 0) {
    lastFail.value = AGENTS[panel.kind]?.cmd || 'zsh'
    logActivity(`Process ${AGENTS[panel.kind]?.label || 'Terminal'} exited with code ${code}`, 'error')
  } else {
    logActivity(`Process ${AGENTS[panel.kind]?.label || 'Terminal'} finished cleanly`, 'info')
  }
}

async function refreshGit() {
  const bridge = window.vibedeck
  if (active.value?.cwd) {
    branch.value = await bridge.gitBranch(active.value.cwd)
  } else {
    branch.value = await bridge.gitBranch()
  }
}

function tick() {
  const now = new Date()
  clock.value = now.toLocaleTimeString('id-ID', { hour12: false }).replace(/\./g, ':')
  dateStr.value = now.toLocaleDateString('id-ID', { weekday: 'short', day: 'numeric', month: 'short' })
}

function handleMenu(m) {
  if (m.action === 'palette') {
    showPalette.value = true
  } else if (m.action === 'shortcuts') {
    showShortcuts.value = true
  } else if (m.action === 'files') {
    showFileManager.value = !showFileManager.value
  } else if (m.view) {
    view.value = m.view
  }
}

function handleKeydown(e) {
  const isMeta = e.metaKey || e.ctrlKey

  if (isMeta && e.key.toLowerCase() === 'e') {
    e.preventDefault()
    showFileManager.value = !showFileManager.value
    return
  }

  if (isMeta && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    showPalette.value = !showPalette.value
    return
  }

  if (isMeta && e.key.toLowerCase() === 't' && !e.shiftKey) {
    e.preventDefault()
    add('terminal')
    return
  }

  if (isMeta && e.key.toLowerCase() === 't' && e.shiftKey) {
    e.preventDefault()
    addSession()
    return
  }

  if (
    e.key === '?' &&
    !showPalette.value &&
    !showPathModal.value &&
    e.target.tagName !== 'INPUT' &&
    e.target.tagName !== 'TEXTAREA' &&
    !e.target.closest?.('.xterm') &&
    !e.target.closest?.('.term-container')
  ) {
    e.preventDefault()
    showShortcuts.value = !showShortcuts.value
    return
  }

  if (e.key === 'Escape') {
    if (showPalette.value) showPalette.value = false
    if (showShortcuts.value) showShortcuts.value = false
    if (showPathModal.value) showPathModal.value = false
    if (maximizedPanelId.value) maximizedPanelId.value = null
    return
  }

  if (isMeta && e.key >= '1' && e.key <= '9') {
    const idx = parseInt(e.key, 10) - 1
    if (sessions.value[idx]) {
      e.preventDefault()
      activeId.value = sessions.value[idx].id
    }
  }
}

watch(activeId, () => {
  maximizedPanelId.value = null
  refreshGit()
})

const PALETTE_ACTIONS = [
  { id: 'term', title: 'Launch Terminal', desc: 'Open interactive zsh shell in workspace path', icon: 'terminal', action: () => add('terminal'), shortcut: '⌘T' },
  { id: 'toggle_files', title: 'Toggle File Manager', desc: 'Open lightweight Kitty-style file manager drawer', icon: 'files', action: () => { showFileManager.value = !showFileManager.value }, shortcut: '⌘E' },
  { id: 'claude', title: 'Launch Claude Code', desc: 'Start Anthropic Claude CLI Agent', icon: 'sparkles', action: () => add('claude'), shortcut: 'Claude' },
  { id: 'codex', title: 'Launch Codex', desc: 'Start OpenAI Codex CLI Agent', icon: 'code', action: () => add('codex'), shortcut: 'Codex' },
  { id: 'kimi', title: 'Launch Kimi Code', desc: 'Start Kimi CLI Agent', icon: 'zap', action: () => add('kimi'), shortcut: 'Kimi' },
  { id: 'gemini', title: 'Launch Gemini', desc: 'Start Google Gemini AI CLI', icon: 'brain', action: () => add('gemini'), shortcut: 'Gemini' },
  { id: 'agy', title: 'Launch AGY Agent', desc: 'Start Google Antigravity CLI Agent', icon: 'bot', action: () => add('agy'), shortcut: 'AGY' },
  { id: 'path_edit', title: 'Set Workspace Directory', desc: 'Configure default opening path for terminals in this session', icon: 'folder', action: () => openPathEditor() },
  { id: 'theme_cycle', title: 'Theme: Cycle Next Theme', desc: 'Switch Dark / Light / Midnight / Matrix / Nord', icon: 'palette', action: () => cycleTheme() },
  { id: 'theme_dark', title: 'Theme: Dark Cyberpunk', desc: 'Switch to Default Dark Sci-Fi theme', icon: 'moon', action: () => setTheme('dark') },
  { id: 'theme_light', title: 'Theme: Clean Light Mode', desc: 'Switch to Crisp Modern Light mode', icon: 'sun', action: () => setTheme('light') },
  { id: 'theme_midnight', title: 'Theme: Midnight Violet', desc: 'Switch to Deep Purple OLED theme', icon: 'sparkles', action: () => setTheme('midnight') },
  { id: 'theme_matrix', title: 'Theme: Matrix Green', desc: 'Switch to Hacker Emerald Matrix theme', icon: 'terminal', action: () => setTheme('matrix') },
  { id: 'theme_nord', title: 'Theme: Nordic Frost', desc: 'Switch to Arctic Twilight Nord theme', icon: 'cloud', action: () => setTheme('nord') },
  { id: 'newsess', title: 'New Workspace Session', desc: 'Create separate tab session (defaults to /)', icon: 'plus', action: () => addSession(), shortcut: '⌘⇧T' },
  { id: 'view_grid', title: 'Layout: Auto Grid', desc: 'Equal distribution grid layout', icon: 'grid', action: () => { layoutMode.value = 'grid' } },
  { id: 'view_cols', title: 'Layout: Columns Split', desc: 'Vertical split columns', icon: 'columns', action: () => { layoutMode.value = 'columns' } },
  { id: 'view_rows', title: 'Layout: Stacked Rows', desc: 'Horizontal split rows', icon: 'rows', action: () => { layoutMode.value = 'stack' } },
]

const filteredActions = computed(() => {
  const q = paletteSearch.value.trim().toLowerCase()
  if (!q) return PALETTE_ACTIONS
  return PALETTE_ACTIONS.filter((a) => a.title.toLowerCase().includes(q) || a.desc.toLowerCase().includes(q))
})

function executePalette(a) {
  showPalette.value = false
  paletteSearch.value = ''
  a.action()
  if (['term', 'claude', 'codex', 'kimi', 'gemini', 'agy', 'newsess', 'view_grid', 'view_cols', 'view_rows'].includes(a.id)) {
    view.value = 'terminal'
  }
}

onMounted(async () => {
  setTheme(currentTheme.value)
  add('terminal')
  if (typeof window !== 'undefined' && window.location.search.includes('preview')) {
    if (active.value) active.value.cwd = '~/Code/vibedeck'
    add('claude')
    add('codex')
    showFileManager.value = true
  }
  refreshGit()
  tick()
  timer = setInterval(() => {
    tick()
    refreshGit()
  }, 3000)
  window.addEventListener('keydown', handleKeydown)
  logActivity('VibeDeck cockpit initialized', 'info')
})

onBeforeUnmount(() => {
  clearInterval(timer)
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="shell">
    <!-- Sleek Vertical Sidebar -->
    <nav class="sidebar" :class="{ wide: expanded }">
      <div class="brand-badge">
        <div class="brand-gem">
          <span class="gem-pulse"></span>
        </div>
        <span class="brand-text">VIBEDECK</span>
      </div>

      <div class="nav-group">
        <button
          v-for="m in MENU"
          :key="m.id"
          class="nav-btn"
          :class="{ active: (m.view && view === m.view) || (m.action === 'palette' && showPalette) }"
          :title="m.label"
          @click="handleMenu(m)"
        >
          <div class="icon-wrap">
            <svg viewBox="0 0 24 24" v-html="ICONS[m.icon]"></svg>
          </div>
          <span class="nav-label">{{ m.label }}</span>
        </button>
      </div>

      <span class="spacer"></span>

      <div class="nav-group foot">
        <button
          v-for="m in FOOT"
          :key="m.id"
          class="nav-btn"
          :class="{ active: m.view && view === m.view }"
          :title="m.label"
          @click="handleMenu(m)"
        >
          <div class="icon-wrap">
            <svg viewBox="0 0 24 24" v-html="ICONS[m.icon]"></svg>
          </div>
          <span class="nav-label">{{ m.label }}</span>
        </button>

        <button
          class="nav-btn toggle"
          :title="expanded ? 'Ciutkan Sidebar' : 'Perluas Sidebar'"
          @click="expanded = !expanded"
        >
          <div class="icon-wrap">
            <svg viewBox="0 0 24 24" v-html="ICONS[expanded ? 'collapse' : 'expand']"></svg>
          </div>
          <span class="nav-label">Ciutkan</span>
        </button>
      </div>
    </nav>

    <!-- Main Workspace Area -->
    <div class="main-content">
      <!-- High-tech Top Header -->
      <header class="topbar">
        <div class="topbar-left">
          <div class="title-cluster">
            <span class="title-main">VIBE</span>
            <span class="title-sub">DECK</span>
            <span class="mode-pill">COCKPIT</span>
          </div>

          <div class="ws-pill" :title="`Workspace Directory: ${active?.cwd || '~'} (Klik untuk ubah path default)`" @click="openPathEditor">
            <svg class="pill-ico" viewBox="0 0 24 24" v-html="ICONS.folder"></svg>
            <span class="ws-path">{{ active?.cwd === '~' ? '~ (Home)' : (active?.cwd === '/' ? '/ (Root)' : active?.cwd) }}</span>
            <span class="ws-edit-btn" title="Ubah Path">
              <svg viewBox="0 0 24 24" v-html="ICONS.edit"></svg>
            </span>
          </div>

          <div class="git-pill" :title="branch ? `Git Branch: ${branch} (Klik untuk refresh)` : 'Git tidak terdeteksi'" @click="refreshGit">
            <svg class="pill-ico" viewBox="0 0 24 24" v-html="ICONS.branch"></svg>
            <span class="git-name">{{ branch ?? 'no-git' }}</span>
          </div>

          <button
            class="files-toggle-btn"
            :class="{ active: showFileManager }"
            :title="showFileManager ? 'Tutup File Manager (⌘E)' : 'Buka File Manager (⌘E)'"
            @click="showFileManager = !showFileManager"
          >
            <svg class="files-btn-ico" viewBox="0 0 24 24" v-html="ICONS.files"></svg>
            <span>Files</span>
            <kbd class="files-kbd">⌘E</kbd>
          </button>
        </div>

        <div class="topbar-center">
          <button class="cmd-trigger" @click="showPalette = true">
            <svg class="cmd-ico" viewBox="0 0 24 24" v-html="ICONS.search"></svg>
            <span class="cmd-text">Quick Launch Agent & Action...</span>
            <kbd class="cmd-kbd">⌘K</kbd>
          </button>
        </div>

        <div class="topbar-right">
          <button class="theme-pill" :title="`Tema saat ini: ${currentTheme.toUpperCase()} (Klik untuk ganti tema)`" @click="cycleTheme">
            <svg class="theme-pill-ico" viewBox="0 0 24 24" v-html="ICONS[THEMES.find(t => t.id === currentTheme)?.icon || 'palette']"></svg>
            <span>{{ THEMES.find(t => t.id === currentTheme)?.label || 'Theme' }}</span>
          </button>

          <div class="telemetry-pill">
            <span class="dot live"></span>
            <span class="telem-label">AGENTS</span>
            <span class="telem-val">{{ runningAgents }}</span>
          </div>

          <div class="telemetry-pill">
            <span class="telem-label">TERMS</span>
            <span class="telem-val">{{ total }}</span>
          </div>

          <div class="clock-display" :title="dateStr">
            <span class="clock-time">{{ clock }}</span>
          </div>
        </div>
      </header>

      <!-- Multi-Session Tab Bar & Controls -->
      <div class="session-bar">
        <div class="tabs-scroll">
          <div
            v-for="(x, i) in sessions"
            :key="x.id"
            class="session-tab"
            :class="{ active: x.id === activeId }"
            @click="activeId = x.id"
            @dblclick="startRename(x)"
          >
            <span class="tab-idx">{{ i + 1 }}</span>

            <input
              v-if="editingSessionId === x.id"
              v-model="editSessionName"
              class="tab-input"
              autofocus
              @blur="saveRename(x)"
              @keydown.enter="saveRename(x)"
              @keydown.esc="editingSessionId = null"
              @click.stop
            />
            <span v-else class="tab-name">{{ x.name }}</span>

            <span class="tab-count">{{ x.panels.length }}</span>

            <button
              v-if="sessions.length > 1"
              class="tab-close"
              title="Tutup Sesi"
              @click.stop="closeSession(x.id)"
            >
              <svg viewBox="0 0 24 24" v-html="ICONS.x"></svg>
            </button>
          </div>

          <button class="add-session-btn" title="Buat Sesi Baru (⌘⇧T)" @click="addSession">
            <svg viewBox="0 0 24 24" v-html="ICONS.plus"></svg>
            <span>Sesi Baru</span>
          </button>
        </div>

        <div class="layout-controls">
          <span class="layout-label">LAYOUT</span>
          <button
            class="layout-btn"
            :class="{ active: layoutMode === 'grid' }"
            title="Auto Grid Layout"
            @click="layoutMode = 'grid'"
          >
            <svg viewBox="0 0 24 24" v-html="ICONS.grid"></svg>
          </button>
          <button
            class="layout-btn"
            :class="{ active: layoutMode === 'columns' }"
            title="Columns Split Layout"
            @click="layoutMode = 'columns'"
          >
            <svg viewBox="0 0 24 24" v-html="ICONS.columns"></svg>
          </button>
          <button
            class="layout-btn"
            :class="{ active: layoutMode === 'stack' }"
            title="Stacked Rows Layout"
            @click="layoutMode = 'stack'"
          >
            <svg viewBox="0 0 24 24" v-html="ICONS.rows"></svg>
          </button>
        </div>
      </div>

      <!-- Vibe Coder Action & Quick Launch Strip -->
      <div class="action-strip">
        <div class="launchers-group">
          <button class="launch-btn term" title="Spawn Zsh Terminal (⌘T)" @click="add('terminal')">
            <svg viewBox="0 0 24 24" v-html="ICONS.terminal"></svg>
            <span>+ Terminal</span>
          </button>

          <button class="launch-btn claude" title="Launch Anthropic Claude Code Agent" @click="add('claude')">
            <svg viewBox="0 0 24 24" v-html="ICONS.sparkles"></svg>
            <span>Claude</span>
          </button>

          <button class="launch-btn codex" title="Launch OpenAI Codex Agent" @click="add('codex')">
            <svg viewBox="0 0 24 24" v-html="ICONS.code"></svg>
            <span>Codex</span>
          </button>

          <button class="launch-btn kimi" title="Launch Kimi Code CLI Agent" @click="add('kimi')">
            <svg viewBox="0 0 24 24" v-html="ICONS.zap"></svg>
            <span>Kimi</span>
          </button>

          <button class="launch-btn gemini" title="Launch Google Gemini AI Agent" @click="add('gemini')">
            <svg viewBox="0 0 24 24" v-html="ICONS.brain"></svg>
            <span>Gemini</span>
          </button>

          <button class="launch-btn agy" title="Launch Antigravity AI Agent" @click="add('agy')">
            <svg viewBox="0 0 24 24" v-html="ICONS.bot"></svg>
            <span>AGY</span>
          </button>
        </div>

        <span class="spacer"></span>

        <div class="info-chips">
          <div class="chip">
            <span class="chip-k">AI ACTIVE</span>
            <span class="chip-v highlight">{{ aiUsed }}</span>
          </div>

          <div v-if="lastFail" class="chip fail" title="Proses terakhir yang gagal">
            <span class="chip-k">LAST FAIL</span>
            <span class="chip-v">{{ lastFail }}</span>
          </div>
          <div v-else class="chip ok">
            <span class="chip-k">SYSTEM</span>
            <span class="chip-v">READY</span>
          </div>
        </div>
      </div>

      <!-- Agent Mission Control View -->
      <section v-if="view === 'agents'" class="view-agents">
        <div class="view-header">
          <div>
            <h2>Agent Mission Control</h2>
            <p>Monitor dan kendalikan semua agen AI dan terminal yang sedang aktif di semua sesi.</p>
          </div>
          <div class="view-stats">
            <div class="stat-card">
              <span class="stat-num">{{ sessions.length }}</span>
              <span class="stat-lbl">Active Sessions</span>
            </div>
            <div class="stat-card">
              <span class="stat-num">{{ runningAgents }}</span>
              <span class="stat-lbl">Running AI Agents</span>
            </div>
            <div class="stat-card">
              <span class="stat-num">{{ total }}</span>
              <span class="stat-lbl">Total Terminals</span>
            </div>
          </div>
        </div>

        <div class="agent-sessions-list">
          <div v-for="x in sessions" :key="x.id" class="agent-session-card">
            <div class="session-card-header">
              <span class="session-tag">{{ x.name }}</span>
              <span class="panel-count-badge">{{ x.panels.length }} Active Processes</span>
            </div>

            <div v-if="x.panels.length" class="agent-grid">
              <div
                v-for="(p, i) in x.panels"
                :key="p.id"
                class="agent-unit-card"
                :class="{ dead: p.alive === false }"
                @click="jump(x.id)"
              >
                <div class="unit-top">
                  <div class="unit-badge" :style="{ background: AGENTS[p.kind]?.badgeBg, color: AGENTS[p.kind]?.accent }">
                    #{{ i + 1 }} {{ AGENTS[p.kind]?.label }}
                  </div>
                  <span class="unit-status" :class="{ running: p.alive !== false }">
                    {{ p.alive === false ? 'EXITED' : 'RUNNING' }}
                  </span>
                </div>
                <div class="unit-body">
                  <span class="unit-cmd">{{ AGENTS[p.kind]?.cmd || 'zsh' }}</span>
                  <span class="unit-id">{{ p.id }}</span>
                </div>
                <button class="unit-jump-btn">
                  <span>Buka Terminal →</span>
                </button>
              </div>
            </div>
            <div v-else class="empty-session-notice">
              Tidak ada proses aktif pada sesi ini.
            </div>
          </div>
        </div>
      </section>

      <!-- Activity Logs View -->
      <section v-else-if="view === 'logs'" class="view-logs">
        <div class="view-header">
          <div>
            <h2>Activity Logs</h2>
            <p>Riwayat peristiwa sistem, lifecycle proses, dan pelacakan agent.</p>
          </div>
        </div>
        <div class="logs-container">
          <div v-for="log in activityLogs" :key="log.id" class="log-entry" :class="log.type">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-type-tag">[{{ log.type.toUpperCase() }}]</span>
            <span class="log-msg">{{ log.text }}</span>
          </div>
          <div v-if="!activityLogs.length" class="empty-logs">Belum ada aktivitas tercatat.</div>
        </div>
      </section>

      <!-- Settings View -->
      <section v-else-if="view === 'settings'" class="view-settings">
        <div class="view-header">
          <div>
            <h2>Settings & Environment</h2>
            <p>Konfigurasi ruang kerja, tema antarmuka, dan direktori kerja default.</p>
          </div>
        </div>

        <div class="settings-grid">
          <!-- Theme Picker Card -->
          <div class="settings-card">
            <h3>Appearance & Themes</h3>
            <p class="desc">Pilih tema antarmuka dan warna terminal sesuai preferensi vibe coding Anda.</p>
            <div class="theme-selection-grid">
              <button
                v-for="t in THEMES"
                :key="t.id"
                class="theme-card-option"
                :class="{ active: currentTheme === t.id }"
                @click="setTheme(t.id)"
              >
                <div class="theme-opt-top">
                  <svg class="theme-opt-ico" viewBox="0 0 24 24" v-html="ICONS[t.icon]"></svg>
                  <span class="theme-opt-name">{{ t.label }}</span>
                  <span v-if="currentTheme === t.id" class="active-pill">ACTIVE</span>
                </div>
                <span class="theme-opt-desc">{{ t.desc }}</span>
              </button>
            </div>
          </div>

          <!-- Workspace CWD Card -->
          <div class="settings-card">
            <h3>Workspace Path (Default: /)</h3>
            <p class="desc">Direktori kerja default untuk sesi <strong>{{ active?.name }}</strong> saat membuka terminal baru.</p>
            <div class="setting-value-box">{{ active?.cwd || '/' }}</div>
            <div class="path-quick-actions" style="margin-top: 10px; display: flex; gap: 8px; flex-wrap: wrap;">
              <button class="path-apply-btn" style="padding: 6px 12px;" @click="openPathEditor">
                Ubah Path Sesi Ini...
              </button>
              <button
                v-for="p in PRESET_PATHS"
                :key="p"
                class="preset-chip"
                @click="active.cwd = p; refreshGit(); logActivity(`Updated path to ${p}`, 'info')"
              >
                {{ p === '/' ? '/ (Root)' : p }}
              </button>
            </div>
          </div>

          <div class="settings-card">
            <h3>Git Repository</h3>
            <p class="desc">Branch aktif pada direktori sesi saat ini.</p>
            <div class="setting-value-box">{{ branch ?? 'Git repository not found on this path' }}</div>
          </div>

          <div class="settings-card">
            <h3>Supported AI Agents</h3>
            <p class="desc">CLI agent yang terdaftar dan siap dijalankan.</p>
            <div class="agent-pills-list">
              <span v-for="(v, k) in AGENTS" :key="k" class="agent-pill-info">
                <strong>{{ v.label }}</strong>
                <small>{{ v.cmd || 'Interactive Shell' }}</small>
              </span>
            </div>
          </div>

          <div class="settings-card">
            <h3>About VibeDeck</h3>
            <p class="desc">Multi-terminal AI agent cockpit untuk vibe coder.</p>
            <div class="about-version">
              <span>Version: 0.1.0</span>
              <span>Backend: Tauri 2 (Rust PTY)</span>
              <span>Frontend: Vue 3 + xterm.js</span>
            </div>
          </div>
        </div>
      </section>

      <!-- Main Terminal Panels Grid -->
      <main
        v-for="x in sessions"
        v-show="view === 'terminal' && x.id === activeId"
        :key="x.id"
        class="terminal-grid"
        :class="[
          `layout-${layoutMode}`,
          {
            'has-maximized': maximizedPanelId,
            'is-single': x.panels.length === 1,
            'is-double': x.panels.length === 2,
            'is-dragging': Boolean(dragId),
          }
        ]"
      >
        <TerminalPanel
          v-for="(p, i) in x.panels"
          v-show="!maximizedPanelId || maximizedPanelId === p.id"
          :key="p.id"
          :panel="p"
          :index="i"
          :is-maximized="maximizedPanelId === p.id"
          :theme="currentTheme"
          :cwd="x.cwd || '~'"
          :class="{
            'over-before': overId === p.id && dropPos === 'before',
            'over-after': overId === p.id && dropPos === 'after',
          }"
          :style="{
            minWidth: p.width ? `${p.width}px` : undefined,
            minHeight: p.height ? `${p.height}px` : undefined,
            height: p.height ? `${p.height}px` : undefined,
          }"
          @toggle-maximize="toggleMaximize(p.id)"
          @dragover.prevent="onPanelDragOver(p.id, $event)"
          @dragleave="overId = null"
          @drop.prevent="onPanelDrop(p.id)"
          @dragstart="dragStart"
          @dragend="dragEnd"
          @rename="handlePanelRename"
          @resize="handlePanelResize"
          @close="close"
          @exit="onExit"
        />

        <div v-if="!x.panels.length" class="empty-cockpit">
          <div class="empty-icon-wrap">
            <svg viewBox="0 0 24 24" v-html="ICONS.terminal"></svg>
          </div>
          <h3>Sesi Ini Belum Memiliki Terminal</h3>
          <p>Mulai coding dengan memanggil terminal default atau agent AI favorit Anda.</p>
          <div class="empty-quick-actions">
            <button class="empty-btn term" @click="add('terminal')">
              <svg viewBox="0 0 24 24" v-html="ICONS.terminal"></svg>
              <span>+ Terminal Zsh</span>
            </button>
            <button class="empty-btn claude" @click="add('claude')">
              <svg viewBox="0 0 24 24" v-html="ICONS.sparkles"></svg>
              <span>Claude Code</span>
            </button>
            <button class="empty-btn codex" @click="add('codex')">
              <svg viewBox="0 0 24 24" v-html="ICONS.code"></svg>
              <span>Codex</span>
            </button>
            <button class="empty-btn kimi" @click="add('kimi')">
              <svg viewBox="0 0 24 24" v-html="ICONS.zap"></svg>
              <span>Kimi Code</span>
            </button>
            <button class="empty-btn gemini" @click="add('gemini')">
              <svg viewBox="0 0 24 24" v-html="ICONS.brain"></svg>
              <span>Gemini</span>
            </button>
            <button class="empty-btn agy" @click="add('agy')">
              <svg viewBox="0 0 24 24" v-html="ICONS.bot"></svg>
              <span>AGY</span>
            </button>
          </div>
          <span class="empty-hint">Shortcut: Tekan <kbd>⌘T</kbd> untuk buka terminal baru atau <kbd>⌘K</kbd> untuk Command Palette.</span>
        </div>
      </main>
    </div>

    <!-- Lightweight File Manager Dock (Kitty-Lite) -->
    <FileManager
      :is-open="showFileManager"
      :root-path="active?.cwd || '~'"
      :theme="currentTheme"
      @close="showFileManager = false"
      @open-terminal="handleFmOpenTerminal"
      @set-session-cwd="handleFmSetSessionCwd"
    />

    <!-- Command Palette Modal (⌘K) -->
    <div v-if="showPalette" class="palette-overlay" @click="showPalette = false">
      <div class="palette-modal" @click.stop>
        <div class="palette-search-box">
          <svg class="search-ico" viewBox="0 0 24 24" v-html="ICONS.search"></svg>
          <input
            v-model="paletteSearch"
            class="palette-input"
            placeholder="Cari aksi, jalankan agent, ubah layout..."
            autofocus
            @keydown.esc="showPalette = false"
          />
          <kbd class="esc-kbd" @click="showPalette = false">ESC</kbd>
        </div>

        <div class="palette-list">
          <button
            v-for="a in filteredActions"
            :key="a.id"
            class="palette-item"
            @click="executePalette(a)"
          >
            <div class="p-item-icon">
              <svg viewBox="0 0 24 24" v-html="ICONS[a.icon]"></svg>
            </div>
            <div class="p-item-info">
              <span class="p-item-title">{{ a.title }}</span>
              <span class="p-item-desc">{{ a.desc }}</span>
            </div>
            <kbd v-if="a.shortcut" class="p-item-key">{{ a.shortcut }}</kbd>
          </button>
        </div>
      </div>
    </div>

    <!-- Keyboard Shortcuts Modal (?) -->
    <div v-if="showShortcuts" class="palette-overlay" @click="showShortcuts = false">
      <div class="shortcuts-modal" @click.stop>
        <div class="modal-header">
          <h3>Vibe Coder Keybindings</h3>
          <button class="close-btn" @click="showShortcuts = false">
            <svg viewBox="0 0 24 24" v-html="ICONS.x"></svg>
          </button>
        </div>
        <div class="shortcuts-grid">
          <div class="sc-row"><kbd>⌘ + E</kbd><span>Buka / Tutup File Manager (Kitty-Lite)</span></div>
          <div class="sc-row"><kbd>⌘ + T</kbd><span>Buka Terminal Baru</span></div>
          <div class="sc-row"><kbd>⌘ + ⇧ + T</kbd><span>Buat Workspace Session Baru</span></div>
          <div class="sc-row"><kbd>⌘ + K</kbd><span>Buka Quick Command Palette</span></div>
          <div class="sc-row"><kbd>⌘ + 1 .. 9</kbd><span>Pindah Tab Sesi</span></div>
          <div class="sc-row"><kbd>Double Click Title</kbd><span>Ganti Nama Window Terminal</span></div>
          <div class="sc-row"><kbd>Resize Handle</kbd><span>Tarik Handle Sudut / Sisi untuk Ubah Ukuran</span></div>
          <div class="sc-row"><kbd>Grip Drag</kbd><span>Tukar Posisi Terminal Panel</span></div>
          <div class="sc-row"><kbd>Esc</kbd><span>Tutup Modal / Pulihkan Layar Penuh</span></div>
        </div>
      </div>
    </div>

    <!-- Workspace Path Customizer Modal -->
    <div v-if="showPathModal" class="palette-overlay" @click="showPathModal = false">
      <div class="path-modal" @click.stop>
        <div class="modal-header">
          <h3>Atur Path Workspace ({{ active?.name }})</h3>
          <button class="close-btn" @click="showPathModal = false">
            <svg viewBox="0 0 24 24" v-html="ICONS.x"></svg>
          </button>
        </div>
        <p>Tentukan direktori kerja default saat membuka terminal baru pada sesi workspace ini.</p>

        <div class="path-input-group">
          <input
            v-model="customPathInput"
            class="path-input"
            placeholder="/ atau ~ atau /Users/..."
            autofocus
            @keydown.enter="saveWorkspacePath"
            @keydown.esc="showPathModal = false"
          />
          <button class="path-apply-btn" @click="saveWorkspacePath">Terapkan</button>
        </div>

        <div v-if="pathError" style="color: var(--red); font-size: 11px; margin-bottom: 12px; font-family: var(--font-mono);">
          ⚠️ {{ pathError }}
        </div>

        <div class="path-presets-label">Pilihan Cepat:</div>
        <div class="path-presets">
          <button
            v-for="p in PRESET_PATHS"
            :key="p"
            class="preset-chip"
            @click="selectPresetPath(p)"
          >
            {{ p === '/' ? '/ (Root)' : p }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
