<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { AGENTS } from '../agents.js'

const props = defineProps({ panel: Object, index: Number })
const emit = defineEmits(['close', 'exit', 'dragstart'])
const el = ref(null)
const alive = ref(true)
const tags = ref([])
const version = ref('')
let term, fit, ro, offData

const agent = AGENTS[props.panel.kind]
const SCAN = [
  [/gemini-[\w.-]+/i, (m) => m[0]],
  [/gpt-[\w.]+(?: (?:low|medium|high))?/i, (m) => m[0]],
  [/(?:Sonnet|Opus|Haiku) [\d.]+/, (m) => m[0]],
  [/no sandbox/i, () => ['no sandbox', 'warn']],
  [/YOLO/, () => ['YOLO', 'warn']],
  [/(?:don't ask on|bypass permissions on)/i, (m) => m[0]],
]
let buf = ''
function scan(d) {
  if (!agent.cmd || tags.value.length >= 4) return
  buf = (buf + d).slice(-4000)
  const plain = buf.replace(/\x1b\[[0-9;?]*[A-Za-z]/g, '')
  for (const [re, fn] of SCAN) {
    const m = plain.match(re)
    if (!m) continue
    const r = fn(m)
    const t = Array.isArray(r) ? { text: r[0], warn: true } : { text: r }
    if (!tags.value.some((x) => x.text === t.text)) tags.value.push(t)
  }
}

onMounted(async () => {
  term = new Terminal({
    fontFamily: 'JetBrains Mono, Menlo, monospace', fontSize: 12, cursorBlink: true,
    scrollback: 5000,
    theme: { background: '#05080f', foreground: '#c9d4e5', cursor: '#22d3ee' },
  })
  fit = new FitAddon()
  term.loadAddon(fit)
  term.open(el.value)
  fit.fit()

  term.onData((d) => window.takara.write(props.panel.id, d))
  if (agent.cmd) window.takara.version(agent.cmd).then((v) => (version.value = v ? `v${v}` : ''))
  window.takara.spawn({
    id: props.panel.id, cmd: agent.cmd, cols: term.cols, rows: term.rows,
  })
  term.onResize(({ cols, rows }) => window.takara.resize(props.panel.id, cols, rows))

  ro = new ResizeObserver(() => fit.fit())
  ro.observe(el.value)
})

function onData(id, d) { if (id === props.panel.id) { term?.write(d); scan(d) } }
function onExit(id, code) {
  if (id !== props.panel.id) return
  alive.value = false
  emit('exit', props.panel, code)
}
window.takara.onData(onData)
window.takara.onExit(onExit)

onBeforeUnmount(() => {
  ro?.disconnect()
  window.takara.kill(props.panel.id)
  term?.dispose()
})
</script>

<template>
  <section class="panel" draggable="false">
    <header class="panel-head" draggable="true" @dragstart="emit('dragstart', panel.id, $event)">
      <span class="badge" :style="{ background: agent.color }">{{ index + 1 }}</span>
      <span class="name">{{ agent.label }} {{ panel.n }}</span>
      <span class="proc">{{ agent.cmd || 'zsh' }} {{ version }}</span>
      <span v-for="t in tags" :key="t.text" class="tag" :class="{ warn: t.warn }">{{ t.text }}</span>
      <span class="spacer"></span>
      <span class="dot" :class="{ off: !alive }"></span>
      <button class="x" title="Tutup" @click="emit('close', panel.id)">✕</button>
    </header>
    <div ref="el" class="term"></div>
  </section>
</template>
