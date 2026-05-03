// Tauri File API extensions
// The `path` property is added by Tauri when dropping files from the OS
interface TauriFile extends File {
  path?: string
}
