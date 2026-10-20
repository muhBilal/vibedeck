import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const isTauri = typeof window !== 'undefined' && Boolean(window.__TAURI_INTERNALS__)

const demoDataListeners = new Set()
const demoExitListeners = new Set()

const bridge = !isTauri ? {
  workspace: async () => '~',
  gitBranch: async () => 'master',
  version: async () => '1.0.0',
  verifyPath: async () => true,
  readDir: async (path) => ({
    current_path: path || '~',
    parent_path: '~',
    entries: [
      { name: 'Documents', path: '~/Documents', is_dir: true, is_symlink: false, size: 0, modified: Date.now() / 1000, ext: '' },
      { name: 'projects', path: '~/Documents/projects', is_dir: true, is_symlink: false, size: 0, modified: Date.now() / 1000, ext: '' },
      { name: 'Code', path: '~/Code', is_dir: true, is_symlink: false, size: 0, modified: Date.now() / 1000, ext: '' },
      { name: 'Desktop', path: '~/Desktop', is_dir: true, is_symlink: false, size: 0, modified: Date.now() / 1000, ext: '' },
      { name: 'package.json', path: '~/package.json', is_dir: false, is_symlink: false, size: 1024, modified: Date.now() / 1000, ext: 'json' },
      { name: 'README.md', path: '~/README.md', is_dir: false, is_symlink: false, size: 2048, modified: Date.now() / 1000, ext: 'md' },
    ],
  }),
  readFilePreview: async () => 'Mock preview content for web demo',
  revealInFinder: async () => {},
  spawn: async ({ id, cmd, cwd }) => {
    setTimeout(() => {
      const banner = `\r\n\x1b[36m◆ VibeDeck Web Demo\x1b[0m\r\n\x1b[90mSession: ${id} | Path: ${cwd || '/'} | Command: ${cmd || 'zsh'} (Browser preview)\x1b[0m\r\n\r\n${cwd || '/'} $ `
      for (const cb of demoDataListeners) cb(id, banner)
    }, 80)
    return 1000
  },
  write: (id, data) => {
    if (data === '\r') {
      for (const cb of demoDataListeners) cb(id, '\r\n/ $ ')
    } else if (data === '\u007f') {
      for (const cb of demoDataListeners) cb(id, '\b \b')
    } else {
      for (const cb of demoDataListeners) cb(id, data)
    }
  },
  resize: () => {},
  kill: (id) => {
    for (const cb of demoExitListeners) cb(id, 0)
  },
  onData: async (cb) => {
    demoDataListeners.add(cb)
    return () => demoDataListeners.delete(cb)
  },
  onExit: async (cb) => {
    demoExitListeners.add(cb)
    return () => demoExitListeners.delete(cb)
  },
} : {
  workspace: () => invoke('get_workspace'),
  gitBranch: (path) => invoke('git_branch', { path }),
  version: (cmd) => invoke('version', { cmd }),
  verifyPath: (path) => invoke('verify_path', { path }),
  readDir: (path) => invoke('read_dir', { path }),
  readFilePreview: (path) => invoke('read_file_preview', { path }),
  revealInFinder: (path) => invoke('reveal_in_finder', { path }),
  spawn: ({ id, cmd, cols, rows, cwd }) => invoke('pty_spawn', { id, cmd, cols, rows, cwd }),
  write: (id, data) => invoke('pty_write', { id, data }),
  resize: (id, cols, rows) => invoke('pty_resize', { id, cols, rows }),
  kill: (id) => invoke('pty_kill', { id }),
  onData: (cb) => listen('pty:data', (e) => cb(...e.payload)),
  onExit: (cb) => listen('pty:exit', (e) => cb(...e.payload)),
}

window.vibedeck = bridge
window.takara = bridge
