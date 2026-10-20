<script setup>
import { computed, onMounted, onBeforeUnmount, ref } from 'vue'
import TerminalPanel from './components/TerminalPanel.vue'
import { AGENTS } from './agents.js'

const workspace = ref('~/Code/takara-space')
const branch = ref(null)
const lastFail = ref('-')
const clock = ref('')
const panels = ref([])
const counters = {}
let uid = 0
let timer

const total = computed(() => panels.value.length)
const aiUsed = computed(() =>
  [...new Set(panels.value.map((p) => p.kind).filter((k) => k !== 'terminal'))]
    .map((k) => AGENTS[k].label[0] + AGENTS[k].label.slice(1).toLowerCase()).join(', ') || '-')
const cols = computed(() => Math.max(1, Math.ceil(Math.sqrt(total.value))))

function add(kind) {
  counters[kind] = (counters[kind] || 0) + 1
  panels.value.push({ id: `p${++uid}`, kind, n: counters[kind] })
}
function close(id) { panels.value = panels.value.filter((p) => p.id !== id) }
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
    <nav class="sidebar">
      <button class="ico active">▣</button>
      <button class="ico">☻</button>
      <button class="ico">✺</button>
      <button class="ico">▤</button>
      <button class="ico">⌕</button>
      <button class="ico">{ }</button>
      <button class="ico">☁</button>
      <span class="spacer"></span>
      <button class="ico">◉</button>
      <button class="ico">⚙</button>
      <button class="ico">»</button>
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
        <div class="tab">1 Session 1 <b>{{ total }}/{{ total }}</b></div>
        <button class="plus">+ ▾</button>
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

      <main class="grid" :style="{ '--cols': cols }">
        <TerminalPanel
          v-for="(p, i) in panels" :key="p.id" :panel="p" :index="i"
          @close="close" @exit="onExit" />
        <div v-if="!total" class="empty">Tidak ada terminal. Tambah lewat tombol di atas.</div>
      </main>
    </div>
  </div>
</template>
