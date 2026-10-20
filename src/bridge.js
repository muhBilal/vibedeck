import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

const isTauri = typeof window !== 'undefined' && Boolean(window.__TAURI_INTERNALS__)

const demoDataListeners = new Set()
const demoExitListeners = new Set()

const bridge = !isTauri ? {
  workspace: async () => '~/Code/vibedeck',
  gitBranch: async () => 'master',
  version: async () => '1.0.0',
  verifyPath: async () => true,
  readDir: async (path) => ({
    current_path: path || '~/Code/vibedeck',
    parent_path: '~/Code',
    entries: [
      { name: 'src', path: '~/Code/vibedeck/src', is_dir: true, is_symlink: false, size: 0, modified: Date.now() / 1000, ext: '' },
      { name: 'src-tauri', path: '~/Code/vibedeck/src-tauri', is_dir: true, is_symlink: false, size: 0, modified: Date.now() / 1000, ext: '' },
      { name: 'docs', path: '~/Code/vibedeck/docs', is_dir: true, is_symlink: false, size: 0, modified: Date.now() / 1000, ext: '' },
      { name: 'package.json', path: '~/Code/vibedeck/package.json', is_dir: false, is_symlink: false, size: 1024, modified: Date.now() / 1000, ext: 'json' },
      { name: 'README.md', path: '~/Code/vibedeck/README.md', is_dir: false, is_symlink: false, size: 2048, modified: Date.now() / 1000, ext: 'md' },
      { name: 'vite.config.js', path: '~/Code/vibedeck/vite.config.js', is_dir: false, is_symlink: false, size: 512, modified: Date.now() / 1000, ext: 'js' },
    ],
  }),
  readFilePreview: async () => 'Mock preview content for web demo',
  revealInFinder: async () => {},
  spawn: async ({ id, cmd, cwd }) => {
    setTimeout(() => {
      let banner = ''
      if (cmd === 'claude') {
        banner = `\r\n\x1b[38;2;249;115;22m✦ Claude Code CLI v1.0.18 (Thinking)\x1b[0m\r\n\x1b[90mAnthropic Claude 3.7 Sonnet | Workspace: ${cwd || '~/Code/vibedeck'}\x1b[0m\r\n\r\n\x1b[32m✔\x1b[0m Architecture indexed. 12 files watched.\r\n\x1b[38;2;249;115;22mclaude >\x1b[0m `
      } else if (cmd === 'codex') {
        banner = `\r\n\x1b[38;2;16;185;129m◆ OpenAI Codex Engine\x1b[0m\r\n\x1b[90mMulti-file coding agent connected to ${cwd || '~/Code/vibedeck'}\x1b[0m\r\n\r\n\x1b[38;2;16;185;129mcodex >\x1b[0m Generating test suites...\r\n`
      } else if (cmd === 'kimi') {
        banner = `\r\n\x1b[38;2;20;184;166m⚡ Kimi Code Agent CLI\x1b[0m\r\n\x1b[90mContext window: 200k tokens | Mode: Deep Research\x1b[0m\r\n\r\n\x1b[38;2;20;184;166mkimi >\x1b[0m `
      } else {
        banner = `\r\n\x1b[38;2;34;211;238m⚡ VibeDeck Cockpit Terminal\x1b[0m \x1b[90m(portable-pty xterm.js)\x1b[0m\r\n\x1b[38;2;168;85;247m~/Code/vibedeck\x1b[0m \x1b[33m(master)\x1b[0m $ `
      }
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
