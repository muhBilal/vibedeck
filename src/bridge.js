import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const demo = !window.__TAURI_INTERNALS__

window.takara = demo ? {
  workspace: async () => '~/Code/takara-space', gitBranch: async () => 'master', version: async () => '1.0.0',
  spawn: async () => 0, write() {}, resize() {}, kill() {}, onData() {}, onExit() {},
} : {
  workspace: () => invoke('get_workspace'),
  gitBranch: () => invoke('git_branch'),
  version: (cmd) => invoke('version', { cmd }),
  spawn: ({ id, cmd, cols, rows }) => invoke('pty_spawn', { id, cmd, cols, rows }),
  write: (id, data) => invoke('pty_write', { id, data }),
  resize: (id, cols, rows) => invoke('pty_resize', { id, cols, rows }),
  kill: (id) => invoke('pty_kill', { id }),
  onData: (cb) => listen('pty:data', (e) => cb(...e.payload)),
  onExit: (cb) => listen('pty:exit', (e) => cb(...e.payload)),
}
