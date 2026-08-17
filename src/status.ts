import type { LauncherStatus } from './types'

export type ViewState = 'checking' | 'needsNode' | 'notInstalled' | 'ready' | 'starting' | 'running' | 'external' | 'error'

export function resolveViewState(status: LauncherStatus | null, loading: boolean): ViewState {
  if (!status || loading) return 'checking'
  if (!status.nodeAvailable) return 'needsNode'
  if (status.error || status.service === 'error') return 'error'
  if (status.service === 'external') return 'external'
  if (status.service === 'running') return 'running'
  if (status.service === 'starting') return 'starting'
  if (!status.installed) return 'notInstalled'
  return 'ready'
}

