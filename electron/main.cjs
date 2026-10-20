const { app, BrowserWindow, ipcMain } = require('electron')
const path = require('path')
const os = require('os')
const { execFile } = require('child_process')
const pty = require('node-pty')

const WORKSPACE = process.env.TAKARA_WORKSPACE || path.join(os.homedir(), 'Code', 'takara-space')
const ptys = new Map()
let win

function createWindow() {
  win = new BrowserWindow({
    width: 1600, height: 960, minWidth: 900, minHeight: 600,
    backgroundColor: '#070b14', title: 'Takara Space',
    webPreferences: { preload: path.join(__dirname, 'preload.cjs'), contextIsolation: true },
  })
  const dev = process.env.VITE_DEV_URL
  dev ? win.loadURL(dev) : win.loadFile(path.join(__dirname, '..', 'dist', 'index.html'))
}

ipcMain.handle('workspace', () => WORKSPACE)

ipcMain.handle('pty:spawn', (_e, { id, cmd, cols, rows }) => {
  const shell = process.env.SHELL || '/bin/zsh'
  const p = pty.spawn(shell, cmd ? ['-lic', `${cmd}; exec ${shell} -l`] : ['-l'], {
    name: 'xterm-256color', cols, rows, cwd: WORKSPACE,
    env: { ...process.env, TERM: 'xterm-256color' },
  })
  ptys.set(id, p)
  p.onData((d) => win?.webContents.send('pty:data', id, d))
  p.onExit(({ exitCode }) => {
    ptys.delete(id)
    win?.webContents.send('pty:exit', id, exitCode)
  })
  return p.pid
})

ipcMain.on('pty:write', (_e, id, d) => ptys.get(id)?.write(d))
ipcMain.on('pty:resize', (_e, id, cols, rows) => {
  try { ptys.get(id)?.resize(Math.max(cols, 2), Math.max(rows, 2)) } catch {}
})
ipcMain.on('pty:kill', (_e, id) => { ptys.get(id)?.kill(); ptys.delete(id) })

ipcMain.handle('git:status', () => new Promise((res) =>
  execFile('git', ['rev-parse', '--abbrev-ref', 'HEAD'], { cwd: WORKSPACE }, (err, out) =>
    res(err ? null : out.trim()))))

app.whenReady().then(createWindow)
app.on('window-all-closed', () => { ptys.forEach((p) => p.kill()); app.quit() })
