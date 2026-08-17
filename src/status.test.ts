import { describe, expect, it } from 'vitest'
import { resolveViewState } from './status'
import type { LauncherStatus } from './types'

const base: LauncherStatus = {
  installed: true,
  installedVersion: '0.1.0',
  runtimeSource: 'managed',
  latestVersion: '0.1.0',
  updateAvailable: false,
  service: 'stopped',
  serviceOrigin: null,
  serviceUrl: 'http://127.0.0.1:3080',
  nodeAvailable: true,
  workspace: '/Users/example/Projects',
  error: null,
}

describe('resolveViewState', () => {
  it('keeps missing runtime separate from a stopped runtime', () => {
    expect(resolveViewState({ ...base, installed: false }, false)).toBe('notInstalled')
    expect(resolveViewState(base, false)).toBe('ready')
  })

  it('surfaces externally-owned services without offering destructive control', () => {
    expect(resolveViewState({ ...base, service: 'external' }, false)).toBe('external')
  })

  it('prioritizes missing Node over package state', () => {
    expect(resolveViewState({ ...base, installed: false, nodeAvailable: false }, false)).toBe('needsNode')
  })
})
