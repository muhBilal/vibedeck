const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('takara', {
  workspace: () => ipcRenderer.invoke('workspace'),
  gitBranch: () => ipcRenderer.invoke('git:status'),
  spawn: (opts) => ipcRenderer.invoke('pty:spawn', opts),
  write: (id, d) => ipcRenderer.send('pty:write', id, d),
  resize: (id, c, r) => ipcRenderer.send('pty:resize', id, c, r),
  kill: (id) => ipcRenderer.send('pty:kill', id),
  onData: (cb) => ipcRenderer.on('pty:data', (_e, id, d) => cb(id, d)),
  onExit: (cb) => ipcRenderer.on('pty:exit', (_e, id, code) => cb(id, code)),
})
