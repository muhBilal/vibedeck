<script setup>
import { onMounted, onBeforeUnmount, ref, watch, nextTick } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { AGENTS } from '../agents.js'
import { ICONS } from '../icons.js'

const props = defineProps({
  panel: Object,
  index: Number,
  isMaximized: Boolean,
  theme: {
    type: String,
    default: 'dark',
  },
  cwd: {
    type: String,
    default: '~',
  },
  fontSize: {
    type: Number,
    default: 13,
  },
})
const emit = defineEmits(['close', 'exit', 'dragstart', 'dragend', 'toggle-maximize', 'rename', 'resize', 'zoom-in', 'zoom-out', 'zoom-reset'])

const el = ref(null)
const tags = ref([])
const version = ref('')
const isEditingTitle = ref(false)
const editTitle = ref('')
const titleInputRef = ref(null)
let term, fit, ro, unData, unExit

const agent = AGENTS[props.panel.kind] || AGENTS.terminal

function startEditingTitle() {
  isEditingTitle.value = true
  editTitle.value = props.panel.title || `${agent.label} ${props.panel.n > 1 ? '(' + props.panel.n + ')' : ''}`
  nextTick(() => {
    titleInputRef.value?.focus()
    titleInputRef.value?.select()
  })
}

function saveTitle() {
  if (editTitle.value.trim()) {
    props.panel.title = editTitle.value.trim()
    emit('rename', props.panel.id, props.panel.title)
  }
  isEditingTitle.value = false
}

function startResize(dir, e) {
  e.preventDefault()
  e.stopPropagation()
  const startX = e.clientX
  const startY = e.clientY
  const panelEl = el.value.closest('.panel')
  const startW = panelEl.getBoundingClientRect().width
  const startH = panelEl.getBoundingClientRect().height

  function onMouseMove(ev) {
    if (dir === 'right' || dir === 'corner') {
      const newW = Math.max(280, Math.round(startW + (ev.clientX - startX)))
      emit('resize', { id: props.panel.id, width: newW })
    }
    if (dir === 'bottom' || dir === 'corner') {
      const newH = Math.max(180, Math.round(startH + (ev.clientY - startY)))
      emit('resize', { id: props.panel.id, height: newH })
    }
    fit?.fit()
  }

  function onMouseUp() {
    window.removeEventListener('mousemove', onMouseMove)
    window.removeEventListener('mouseup', onMouseUp)
    setTimeout(() => fit?.fit(), 40)
  }

  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseup', onMouseUp)
}

function resetSize() {
  emit('resize', { id: props.panel.id, width: null, height: null })
  setTimeout(() => fit?.fit(), 40)
}

const XTERM_THEMES = {
  dark: {
    background: '#090d16',
    foreground: '#e2e8f0',
    cursor: '#22d3ee',
    cursorAccent: '#090d16',
    selectionBackground: 'rgba(34, 211, 238, 0.28)',
    selectionForeground: '#ffffff',
    black: '#0f172a',
    red: '#f87171',
    green: '#34d399',
    yellow: '#fbbf24',
    blue: '#60a5fa',
    magenta: '#c084fc',
    cyan: '#22d3ee',
    white: '#cbd5e1',
    brightBlack: '#475569',
    brightRed: '#fca5a5',
    brightGreen: '#6ee7b7',
    brightYellow: '#fde047',
    brightBlue: '#93c5fd',
    brightMagenta: '#d8b4fe',
    brightCyan: '#67e8f9',
    brightWhite: '#f8fafc',
  },
  light: {
    background: '#ffffff',
    foreground: '#0f172a',
    cursor: '#0284c7',
    cursorAccent: '#ffffff',
    selectionBackground: 'rgba(2, 132, 199, 0.25)',
    selectionForeground: '#0f172a',
    black: '#334155',
    red: '#dc2626',
    green: '#16a34a',
    yellow: '#d97706',
    blue: '#2563eb',
    magenta: '#9333ea',
    cyan: '#0284c7',
    white: '#64748b',
    brightBlack: '#94a3b8',
    brightRed: '#ef4444',
    brightGreen: '#22c55e',
    brightYellow: '#eab308',
    brightBlue: '#3b82f6',
    brightMagenta: '#a855f7',
    brightCyan: '#38bdf8',
    brightWhite: '#0f172a',
  },
  midnight: {
    background: '#090714',
    foreground: '#f1f0f7',
    cursor: '#c084fc',
    cursorAccent: '#090714',
    selectionBackground: 'rgba(192, 132, 252, 0.3)',
    selectionForeground: '#ffffff',
    black: '#141126',
    red: '#f43f5e',
    green: '#34d399',
    yellow: '#fbbf24',
    blue: '#818cf8',
    magenta: '#c084fc',
    cyan: '#a855f7',
    white: '#cbd5e1',
    brightBlack: '#5c5578',
    brightRed: '#fb7185',
    brightGreen: '#6ee7b7',
    brightYellow: '#fde047',
    brightBlue: '#a5b4fc',
    brightMagenta: '#e879f9',
    brightCyan: '#d8b4fe',
    brightWhite: '#ffffff',
  },
  matrix: {
    background: '#040d06',
    foreground: '#e2fce9',
    cursor: '#10b981',
    cursorAccent: '#040d06',
    selectionBackground: 'rgba(16, 185, 129, 0.3)',
    selectionForeground: '#ffffff',
    black: '#07140b',
    red: '#ef4444',
    green: '#10b981',
    yellow: '#eab308',
    blue: '#06b6d4',
    magenta: '#84cc16',
    cyan: '#34d399',
    white: '#80cfa0',
    brightBlack: '#1b4d29',
    brightRed: '#f87171',
    brightGreen: '#4ade80',
    brightYellow: '#fde047',
    brightBlue: '#22d3ee',
    brightMagenta: '#a3e635',
    brightCyan: '#6ee7b7',
    brightWhite: '#ffffff',
  },
  nord: {
    background: '#282e39',
    foreground: '#eceff4',
    cursor: '#88c0d0',
    cursorAccent: '#282e39',
    selectionBackground: 'rgba(136, 192, 208, 0.3)',
    selectionForeground: '#ffffff',
    black: '#3b4252',
    red: '#bf616a',
    green: '#a3be8c',
    yellow: '#ebcb8b',
    blue: '#81a1c1',
    magenta: '#b48ead',
    cyan: '#88c0d0',
    white: '#e5e9f0',
    brightBlack: '#4c566a',
    brightRed: '#d08770',
    brightGreen: '#a3be8c',
    brightYellow: '#ebcb8b',
    brightBlue: '#8fbcbb',
    brightMagenta: '#b48ead',
    brightCyan: '#88c0d0',
    brightWhite: '#eceff4',
  },
}

const SCAN = [
  [/gemini-[\w.-]+/i, (m) => m[0]],
  [/gpt-[\w.]+(?: (?:low|medium|high))?/i, (m) => m[0]],
  [/o[13](?:-mini|-preview)?/i, (m) => m[0]],
  [/(?:Sonnet|Opus|Haiku) [\d.]+/i, (m) => m[0]],
  [/claude-[\w.-]+/i, (m) => m[0]],
  [/deepseek-[\w.-]+/i, (m) => m[0]],
  [/moonshot-[\w.-]+/i, (m) => m[0]],
  [/no sandbox/i, () => ['no sandbox', 'warn']],
  [/YOLO/i, () => ['YOLO', 'warn']],
  [/(?:don't ask on|bypass permissions on)/i, (m) => m[0]],
]

let buf = ''
function scan(d) {
  if (!agent.cmd || tags.value.length >= 5) return
  buf = (buf + d).slice(-4000)
  const plain = buf.replace(/\x1b\[[0-9;?]*[A-Za-z]/g, '')
  for (const [re, fn] of SCAN) {
    const m = plain.match(re)
    if (!m) continue
    const r = fn(m)
    const t = Array.isArray(r) ? { text: r[0], warn: true } : { text: r }
    if (!tags.value.some((x) => x.text.toLowerCase() === t.text.toLowerCase())) {
      tags.value.push(t)
    }
  }
}

function clear() {
  term?.clear()
}

function restart() {
  if (term) {
    term.reset()
    tags.value = []
    buf = ''
    props.panel.alive = true
    const bridge = window.vibedeck
    bridge.kill(props.panel.id)
    bridge.spawn({
      id: props.panel.id,
      cmd: agent.cmd,
      cols: term.cols,
      rows: term.rows,
      cwd: props.panel.cwd || props.cwd || '~',
    })
  }
}

onMounted(async () => {
  const chosenTheme = XTERM_THEMES[props.theme] || XTERM_THEMES.dark
  term = new Terminal({
    fontFamily: "'JetBrainsMono Nerd Font', 'JetBrainsMono NFM', 'CaskaydiaCove Nerd Font', 'FiraCode Nerd Font', 'Symbols Nerd Font', 'JetBrains Mono', 'SF Mono', Menlo, Monaco, Consolas, monospace",
    fontSize: props.fontSize || 13,
    lineHeight: 1.28,
    cursorBlink: true,
    cursorStyle: 'bar',
    cursorWidth: 2,
    scrollback: 10000,
    theme: chosenTheme,
  })

  term.attachCustomKeyEventHandler((e) => {
    if ((e.ctrlKey || e.metaKey) && e.type === 'keydown') {
      if (e.key === '=' || e.key === '+' || e.code === 'NumpadAdd' || e.code === 'Equal') {
        emit('zoom-in')
        return false
      }
      if (e.key === '-' || e.key === '_' || e.code === 'NumpadSubtract' || e.code === 'Minus') {
        emit('zoom-out')
        return false
      }
      if (e.key === '0' || e.code === 'Numpad0' || e.code === 'Digit0') {
        emit('zoom-reset')
        return false
      }
    }
    return true
  })

  fit = new FitAddon()
  term.loadAddon(fit)
  term.open(el.value)
  fit.fit()

  const bridge = window.vibedeck
  term.onData((d) => bridge.write(props.panel.id, d))
  if (agent.cmd) {
    bridge.version(agent.cmd).then((v) => (version.value = v ? `v${v}` : ''))
  }

  bridge.spawn({
    id: props.panel.id,
    cmd: agent.cmd,
    cols: term.cols,
    rows: term.rows,
    cwd: props.panel.cwd || props.cwd || '~',
  })

  term.onResize(({ cols, rows }) => bridge.resize(props.panel.id, cols, rows))

  function refit() {
    if (el.value && el.value.offsetWidth > 0 && el.value.offsetHeight > 0) {
      try {
        fit?.fit()
        term?.refresh(0, (term.rows || 1) - 1)
      } catch (_) {}
    }
  }

  ro = new ResizeObserver(() => {
    refit()
  })
  ro.observe(el.value)
  window.addEventListener('resize', refit)

  unData = await bridge.onData((id, d) => {
    if (id === props.panel.id) {
      term?.write(d)
      scan(d)
    }
  })

  unExit = await bridge.onExit((id, code) => {
    if (id !== props.panel.id) return
    props.panel.alive = false
    emit('exit', props.panel, code)
  })
})

watch(() => props.isMaximized, () => {
  setTimeout(() => {
    try {
      if (el.value && el.value.offsetWidth > 0 && el.value.offsetHeight > 0) {
        fit?.fit()
      }
    } catch (_) {}
  }, 60)
})

watch(() => props.theme, (newTheme) => {
  if (term && XTERM_THEMES[newTheme]) {
    term.options.theme = XTERM_THEMES[newTheme]
  }
})

watch(() => props.fontSize, (newSize) => {
  if (term && newSize) {
    term.options.fontSize = newSize
    nextTick(() => {
      refit()
    })
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', refit)
  ro?.disconnect()
  unData?.()
  unExit?.()
  const bridge = window.vibedeck
  bridge.kill(props.panel.id)
  term?.dispose()
})
</script>

<template>
  <section class="panel" :class="{ 'is-maximized': isMaximized, 'has-custom-size': panel.width || panel.height }" draggable="false">
    <header
      class="panel-head"
      draggable="true"
      @dragstart="emit('dragstart', panel.id, $event)"
      @dragend="emit('dragend', panel.id, $event)"
    >
      <span class="grip" title="Seret untuk pindahkan posisi window">
        <svg viewBox="0 0 24 24" v-html="ICONS.grip"></svg>
      </span>

      <div
        class="badge-tag"
        :style="{ background: agent.badgeBg, borderColor: agent.badgeBorder, color: agent.accent }"
        :title="'Double-click atau klik ikon pensil untuk ganti nama window'"
        @dblclick="startEditingTitle"
      >
        <span class="idx">#{{ index + 1 }}</span>

        <input
          v-if="isEditingTitle"
          ref="titleInputRef"
          v-model="editTitle"
          class="title-edit-input"
          @blur="saveTitle"
          @keydown.enter="saveTitle"
          @keydown.esc="isEditingTitle = false"
          @click.stop
        />
        <span v-else class="kind-name">{{ panel.title || agent.label }}</span>
        <span v-if="!panel.title && panel.n > 1" class="num-n">({{ panel.n }})</span>

        <button v-if="!isEditingTitle" class="edit-title-btn" title="Ganti Nama Window" @click.stop="startEditingTitle">
          <svg viewBox="0 0 24 24" v-html="ICONS.edit"></svg>
        </button>
      </div>

      <div class="proc-pill" :title="agent.desc">
        <span class="proc-cmd">{{ agent.cmd || 'zsh' }}</span>
        <span v-if="version" class="proc-ver">{{ version }}</span>
      </div>

      <div v-if="panel.cwd && panel.cwd !== cwd" class="panel-cwd-badge" :title="`Working directory: ${panel.cwd}`">
        <svg viewBox="0 0 24 24" v-html="ICONS.folder"></svg>
        <span>{{ panel.cwd.replace(/\/Users\/[^\/]+/, '~') }}</span>
      </div>

      <div class="tags-list">
        <span v-for="t in tags" :key="t.text" class="tag" :class="{ warn: t.warn }">
          {{ t.text }}
        </span>
      </div>

      <span class="spacer"></span>

      <div class="status-indicator" :title="panel.alive === false ? 'Proses Selesai / Exited' : 'Proses Aktif (Running)'">
        <span class="dot" :class="{ off: panel.alive === false }"></span>
        <span class="status-text">{{ panel.alive === false ? 'EXITED' : 'RUNNING' }}</span>
      </div>

      <div class="panel-actions">
        <button v-if="panel.alive === false" class="act-btn restart" title="Jalankan Ulang Proses" @click="restart">
          <svg viewBox="0 0 24 24" v-html="ICONS.restart"></svg>
        </button>

        <button class="act-btn" title="Bersihkan Layar Terminal" @click="clear">
          <svg viewBox="0 0 24 24" v-html="ICONS.trash"></svg>
        </button>

        <button class="act-btn" :title="isMaximized ? 'Pulihkan Ukuran Grid' : 'Fokuskan / Maksimalkan Panel'" @click="emit('toggle-maximize', panel.id)">
          <svg viewBox="0 0 24 24" v-html="isMaximized ? ICONS.minimize : ICONS.maximize"></svg>
        </button>

        <button class="act-btn close" title="Tutup Terminal" @click="emit('close', panel.id)">
          <svg viewBox="0 0 24 24" v-html="ICONS.x"></svg>
        </button>
      </div>
    </header>

    <div ref="el" class="term-container" @click="term?.focus()"></div>

    <!-- Interactive Drag-to-Resize Handles -->
    <div
      v-if="!isMaximized"
      class="resize-handle right"
      title="Tarik untuk ubah lebar terminal"
      @mousedown="startResize('right', $event)"
    ></div>
    <div
      v-if="!isMaximized"
      class="resize-handle bottom"
      title="Tarik untuk ubah tinggi terminal"
      @mousedown="startResize('bottom', $event)"
    ></div>
    <div
      v-if="!isMaximized"
      class="resize-handle corner"
      title="Tarik untuk ubah ukuran (Double-click untuk reset auto)"
      @mousedown="startResize('corner', $event)"
      @dblclick="resetSize"
    ></div>
  </section>
</template>
