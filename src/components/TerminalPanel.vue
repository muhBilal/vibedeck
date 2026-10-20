<script setup>
import { onMounted, onBeforeUnmount, ref } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { AGENTS } from '../agents.js'

const props = defineProps({ panel: Object, index: Number })
const emit = defineEmits(['close', 'exit'])
const el = ref(null)
const alive = ref(true)
let term, fit, ro, offData

const agent = AGENTS[props.panel.kind]

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
  window.takara.spawn({
    id: props.panel.id, cmd: agent.cmd, cols: term.cols, rows: term.rows,
  })
  term.onResize(({ cols, rows }) => window.takara.resize(props.panel.id, cols, rows))

  ro = new ResizeObserver(() => fit.fit())
  ro.observe(el.value)
})

function onData(id, d) { if (id === props.panel.id) term?.write(d) }
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
  <section class="panel">
    <header class="panel-head">
      <span class="badge" :style="{ background: agent.color }">{{ index + 1 }}</span>
      <span class="name">{{ agent.label }} {{ panel.n }}</span>
      <span class="proc">{{ agent.cmd || 'zsh' }}</span>
      <span class="spacer"></span>
      <span class="dot" :class="{ off: !alive }"></span>
      <button class="x" title="Tutup" @click="emit('close', panel.id)">✕</button>
    </header>
    <div ref="el" class="term"></div>
  </section>
</template>
