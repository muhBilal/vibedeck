import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

window.takara = {
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
