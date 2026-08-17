export type ThemePreference = 'system' | 'light' | 'dark'
export type Locale = 'zh-CN' | 'en'
export type ServiceState = 'stopped' | 'starting' | 'running' | 'external' | 'error'
export type RuntimeSource = 'managed' | 'global' | 'npx' | null
export type ServiceOrigin = 'launcher' | 'terminal' | 'unknown' | null

export interface LauncherStatus {
  installed: boolean
  installedVersion: string | null
  runtimeSource: RuntimeSource
  latestVersion: string | null
  updateAvailable: boolean
  service: ServiceState
  serviceOrigin: ServiceOrigin
  serviceUrl: string
  nodeAvailable: boolean
  workspace: string
  error: string | null
}

export type LauncherAction = 'install' | 'update' | 'start' | 'restart' | 'stop'
