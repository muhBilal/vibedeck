<script setup>
import { computed, onMounted, onBeforeUnmount, ref } from 'vue'
import TerminalPanel from './components/TerminalPanel.vue'
import { AGENTS } from './agents.js'
import { ICONS } from './icons.js'

const workspace = ref('~/Code/takara-space')
const branch = ref(null)
const lastFail = ref('-')
const clock = ref('')
const sessions = ref([{ id: 1, name: 'Session 1', panels: [], counters: {} }])
const activeId = ref(1)
const view = ref('terminal')
const expanded = ref(false)
const MENU = [
  { id: 'terminal', icon: 'terminal', label: 'Terminal', view: 'terminal' },
  { id: 'agents', icon: 'bot', label: 'Agents', view: 'agents' },
  { id: 'memory', icon: 'brain', label: 'Memory' },
  { id: 'logs', icon: 'doc', label: 'Logs' },
  { id: 'search', icon: 'search', label: 'Search' },
  { id: 'code', icon: 'code', label: 'Code' },
  { id: 'cloud', icon: 'cloud', label: 'Cloud' },
]
const FOOT = [
  { id: 'profile', icon: 'user', label: 'Profile' },
  { id: 'settings', icon: 'gear', label: 'Settings' },
]
let uid = 0
let sid = 1
const active = computed(() => sessions.value.find((x) => x.id === activeId.value))
const allPanels = computed(() => sessions.value.flatMap((x) => x.panels))
let timer

const total = computed(() => allPanels.value.length)
const aiUsed = computed(() =>
  [...new Set(allPanels.value.map((p) => p.kind).filter((k) => k !== 'terminal'))]
    .map((k) => AGENTS[k].label[0] + AGENTS[k].label.slice(1).toLowerCase()).join(', ') || '-')

function add(kind) {
  const c = active.value.counters
  c[kind] = (c[kind] || 0) + 1
  active.value.panels.push({ id: `p${++uid}`, kind, n: c[kind] })
}
function addSession() {
  sessions.value.push({ id: ++sid, name: `Session ${sid}`, panels: [], counters: {} })
  activeId.value = sid
  add('terminal')
}
function closeSession(id) {
  if (sessions.value.length === 1) return
  const i = sessions.value.findIndex((x) => x.id === id)
  sessions.value.splice(i, 1)
  if (activeId.value === id) activeId.value = sessions.value[Math.max(0, i - 1)].id
}
let dragId = null
const overId = ref(null)
function dragStart(id, e) { dragId = id; e.dataTransfer.effectAllowed = 'move'; e.dataTransfer.setData('text/plain', id) }
function drop(id) {
  overId.value = null
  const list = active.value.panels
  const from = list.findIndex((p) => p.id === dragId)
  const to = list.findIndex((p) => p.id === id)
  if (from < 0 || to < 0 || from === to) return
  const [m] = list.splice(from, 1)
  list.splice(to, 0, m)
}
function jump(sessionId) { activeId.value = sessionId; view.value = 'terminal' }
function close(id) { active.value.panels = active.value.panels.filter((p) => p.id !== id) }
function onExit(panel, code) {
  if (code !== 0) lastFail.value = AGENTS[panel.kind].cmd || 'zsh'
}

async function refreshGit() { branch.value = await window.takara.gitBranch() }
const tick = () => { clock.value = new Date().toLocaleTimeString('id-ID', { hour12: false }).replace(/\./g, ':') }

onMounted(async () => {
  workspace.value = await window.takara.workspace()
  add('terminal')
  refreshGit()
  tick()
  timer = setInterval(() => { tick(); refreshGit() }, 1000)
})
onBeforeUnmount(() => clearInterval(timer))
</script>

<template>
  <div class="shell">
    <nav class="sidebar" :class="{ wide: expanded }">
      <button v-for="m in MENU" :key="m.id" class="ico" :class="{ active: m.view && view === m.view }"
        :title="m.view ? m.label : `${m.label} (belum aktif)`" @click="m.view && (view = m.view)">
        <svg viewBox="0 0 24 24" v-html="ICONS[m.icon]"></svg><span class="lbl">{{ m.label }}</span>
      </button>
      <span class="spacer"></span>
      <button v-for="m in FOOT" :key="m.id" class="ico" :title="`${m.label} (belum aktif)`">
        <svg viewBox="0 0 24 24" v-html="ICONS[m.icon]"></svg><span class="lbl">{{ m.label }}</span>
      </button>
      <button class="ico" :title="expanded ? 'Ciutkan' : 'Perluas'" @click="expanded = !expanded">
        <svg viewBox="0 0 24 24" v-html="ICONS[expanded ? 'collapse' : 'expand']"></svg><span class="lbl">Ciutkan</span>
      </button>
    </nav>

    <div class="main">
      <header class="top">
        <div class="logo">TAKARA SPACE</div>
        <div class="ws"><small>WORKSPACE</small><span>{{ workspace }}</span></div>
        <div class="notice">Settings is not wired yet. Open workspace is ready now.</div>
        <div class="sys"><small>SYSTEM STATUS</small><span class="ok">✓ SECURE</span></div>
        <div class="clock">{{ clock }}</div>
      </header>

      <div class="sessions">
        <div v-for="(x, i) in sessions" :key="x.id" class="tab" :class="{ on: x.id === activeId }"
          @click="activeId = x.id">
          {{ i + 1 }} {{ x.name }} <b>{{ x.panels.length }}/{{ x.panels.length }}</b>
          <button v-if="sessions.length > 1" class="tx" @click.stop="closeSession(x.id)">✕</button>
        </div>
        <button class="plus" title="Sesi baru" @click="addSession">+ ▾</button>
      </div>

      <div class="chips">
        <span class="chip"><small>PRESET</small> Custom</span>
        <span class="chip"><small>AI</small> {{ aiUsed }}</span>
        <span class="chip"><small>TERMS</small> {{ total }}/{{ total }}</span>
        <span class="chip"><small>GIT</small> {{ branch ?? 'Git unavailable' }}</span>
        <span class="chip"><small>LAST FAIL</small> <em class="fail">{{ lastFail }}</em></span>
        <span class="spacer"></span>
        <button class="quick" @click="add('terminal')">+ Terminal</button>
        <button class="quick" @click="add('codex')">Codex</button>
        <button class="quick" @click="add('claude')">Claude</button>
        <button class="quick" @click="add('gemini')">Gemini</button>
      </div>

      <section v-if="view === 'agents'" class="agents">
        <div v-for="x in sessions" :key="x.id">
          <h4>{{ x.name }}</h4>
          <button v-for="(p, i) in x.panels" :key="p.id" class="arow" @click="jump(x.id)">
            <span class="dot" :class="{ off: p.alive === false }"></span>
            <span class="aname">{{ AGENTS[p.kind].label }} {{ p.n }}</span>
            <span class="proc">{{ AGENTS[p.kind].cmd || 'zsh' }} · #{{ i + 1 }}</span>
            <span class="proc">{{ p.alive === false ? 'EXITED' : 'RUNNING' }}</span>
          </button>
        </div>
      </section>

      <main v-for="x in sessions" v-show="view === 'terminal' && x.id === activeId" :key="x.id" class="grid"
        :style="{ '--cols': Math.max(1, Math.ceil(Math.sqrt(x.panels.length))) }">
        <TerminalPanel
          v-for="(p, i) in x.panels" :key="p.id" :panel="p" :index="i"
          :class="{ over: overId === p.id }"
          @dragover.prevent="overId = p.id" @dragleave="overId = null" @drop.prevent="drop(p.id)"
          @dragstart="dragStart" @close="close" @exit="onExit" />
        <div v-if="!x.panels.length" class="empty">Tidak ada terminal. Tambah lewat tombol di atas.</div>
      </main>
    </div>
  </div>
</template>
