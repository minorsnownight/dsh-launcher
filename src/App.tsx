import { useCallback, useEffect, useMemo, useState } from 'react'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { LogicalSize } from '@tauri-apps/api/dpi'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { marked, Renderer } from 'marked'
import {
  ArrowUpRight,
  Check,
  ChevronRight,
  CircleAlert,
  Folder,
  Globe2,
  LoaderCircle,
  Moon,
  Play,
  RefreshCw,
  RotateCcw,
  Settings2,
  Sparkles,
  Square,
  Sun,
  X,
} from 'lucide-react'
import { translate, type MessageKey } from './i18n'
import { resolveViewState, type ViewState } from './status'
import type { ChangelogInfo, LauncherAction, LauncherStatus, Locale, ThemePreference } from './types'

const emptyStatus: LauncherStatus = {
  installed: false,
  installedVersion: null,
  runtimeSource: null,
  latestVersion: null,
  updateAvailable: false,
  service: 'stopped',
  serviceOrigin: null,
  serviceUrl: 'http://127.0.0.1:3080',
  nodeAvailable: true,
  workspace: '',
  error: null,
}

function readLocale(): Locale {
  const saved = localStorage.getItem('locale')
  if (saved === 'en' || saved === 'zh-CN') return saved
  return navigator.language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en'
}

function readTheme(): ThemePreference {
  const saved = localStorage.getItem('theme')
  return saved === 'light' || saved === 'dark' || saved === 'system' ? saved : 'system'
}

function stateCopy(view: ViewState): { title: MessageKey; description: MessageKey } {
  const copy: Record<ViewState, { title: MessageKey; description: MessageKey }> = {
    checking: { title: 'checking', description: 'checkingDescription' },
    needsNode: { title: 'needsNode', description: 'needsNodeDescription' },
    notInstalled: { title: 'notInstalled', description: 'notInstalledDescription' },
    ready: { title: 'ready', description: 'readyDescription' },
    starting: { title: 'starting', description: 'startingDescription' },
    running: { title: 'running', description: 'runningDescription' },
    external: { title: 'external', description: 'externalDescription' },
    error: { title: 'error', description: 'checkingDescription' },
  }
  return copy[view]
}

const changelogRenderer = new Renderer()
changelogRenderer.html = ({ text }) => {
  const heading = text.trim().match(/^<h([1-6])(?:\s+id="[-\w.]+")?>([^<]*)<\/h\1>$/i)
  if (!heading) return ''
  const level = heading[1]
  const label = heading[2]
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
  return `<h${level}>${label}</h${level}>`
}
marked.setOptions({ breaks: true, gfm: true, renderer: changelogRenderer })

function localizedReleaseNotes(body: string, locale: Locale): string {
  const sections = body.split(/\r?\n---\r?\n/)
  if (sections.length < 2) return body
  if (locale === 'zh-CN') {
    return sections[0].replace(/^\[中文\].*\r?\n+/, '')
  }
  return sections[1]
}

function formatDate(iso: string | null, locale: Locale): string | null {
  if (!iso) return null
  try {
    const date = new Date(iso)
    return date.toLocaleDateString(locale === 'zh-CN' ? 'zh-CN' : 'en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    })
  } catch {
    return null
  }
}

function App() {
  const [locale, setLocale] = useState<Locale>(readLocale)
  const [theme, setTheme] = useState<ThemePreference>(readTheme)
  const [status, setStatus] = useState<LauncherStatus | null>(null)
  const [checking, setChecking] = useState(true)
  const [busy, setBusy] = useState<LauncherAction | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [preferencesOpen, setPreferencesOpen] = useState(false)
  const [changelogOpen, setChangelogOpen] = useState(false)
  const [changelog, setChangelog] = useState<ChangelogInfo | null>(null)
  const [changelogLoading, setChangelogLoading] = useState(false)
  const [changelogError, setChangelogError] = useState<string | null>(null)
  const [updating, setUpdating] = useState(false)

  const t = useCallback((key: MessageKey) => translate(locale, key), [locale])

  const refresh = useCallback(async () => {
    setChecking(true)
    setError(null)
    if (!isTauri()) {
      setStatus({ ...emptyStatus, latestVersion: '0.1.0-rc.6', workspace: '/Users/you/Projects' })
      setChecking(false)
      return
    }
    try {
      setStatus(await invoke<LauncherStatus>('get_status'))
    } catch (reason) {
      setStatus((current) => ({ ...(current ?? emptyStatus), error: String(reason) }))
      setError(String(reason))
    } finally {
      setChecking(false)
    }
  }, [])

  useEffect(() => void refresh(), [refresh])

  useEffect(() => {
    localStorage.setItem('locale', locale)
    document.documentElement.lang = locale
  }, [locale])

  useEffect(() => {
    localStorage.setItem('theme', theme)
    const media = window.matchMedia('(prefers-color-scheme: dark)')
    const apply = () => {
      document.documentElement.dataset.theme = theme === 'system' ? (media.matches ? 'dark' : 'light') : theme
    }
    apply()
    media.addEventListener('change', apply)
    return () => media.removeEventListener('change', apply)
  }, [theme])

  useEffect(() => {
    if (!isTauri()) return

    const timer = window.setTimeout(() => {
      const main = document.querySelector('main')
      if (!main) return

      const contentBottom = main.getBoundingClientRect().bottom
      const desiredHeight = Math.max(560, Math.min(720, Math.ceil(contentBottom + 28)))
      if (Math.abs(window.innerHeight - desiredHeight) > 3) {
        void getCurrentWindow().setSize(new LogicalSize(window.innerWidth, desiredHeight))
      }
    }, 80)

    return () => window.clearTimeout(timer)
  }, [busy, checking, error, locale, preferencesOpen, status])

  const view = resolveViewState(status, checking)
  const copy = stateCopy(view)
  const shownStatus = status ?? emptyStatus
  const statusHint = error ?? (view === 'error' || view === 'needsNode' || view === 'external' ? t(copy.description) : null)

  const act = async (action: LauncherAction) => {
    setBusy(action)
    setError(null)
    try {
      setStatus(await invoke<LauncherStatus>('perform_action', { action }))
    } catch (reason) {
      setError(String(reason))
      await refresh()
    } finally {
      setBusy(null)
    }
  }

  const openChangelog = useCallback(async () => {
    if (!shownStatus.latestVersion) return
    setChangelogOpen(true)
    setChangelog(null)
    setChangelogError(null)
    setChangelogLoading(true)
    try {
      const info = await invoke<ChangelogInfo>('fetch_changelog', { version: shownStatus.latestVersion })
      setChangelog(info)
    } catch (reason) {
      setChangelogError(String(reason))
    } finally {
      setChangelogLoading(false)
    }
  }, [shownStatus.latestVersion])

  const performUpdate = useCallback(async () => {
    setUpdating(true)
    setError(null)
    try {
      setStatus(await invoke<LauncherStatus>('perform_action', { action: 'update' }))
      setChangelogOpen(false)
    } catch (reason) {
      setError(String(reason))
      await refresh()
    } finally {
      setUpdating(false)
    }
  }, [refresh])

  const closeChangelog = useCallback(() => {
    if (updating) return
    setChangelogOpen(false)
  }, [updating])

  const handleChangelogClick = useCallback((e: React.MouseEvent<HTMLElement>) => {
    const target = e.target as HTMLElement
    const anchor = target.closest('a')
    if (!anchor) return
    e.preventDefault()
    const href = anchor.getAttribute('href')
    if (href && href.startsWith('http')) {
      void invoke('open_external', { url: href })
    }
  }, [])

  const chooseWorkspace = async () => {
    setError(null)
    try {
      setStatus(await invoke<LauncherStatus>('choose_workspace'))
    } catch (reason) {
      setError(String(reason))
    }
  }

  const primaryAction = useMemo(() => {
    if (checking || busy) return null
    if (view === 'needsNode') {
      return { label: t('needsNode'), icon: ArrowUpRight, run: () => invoke('open_external', { url: 'https://nodejs.org/' }) }
    }
    if (view === 'notInstalled') return { label: t('install'), icon: ArrowUpRight, run: () => act('install') }
    if (view === 'ready') return { label: t('start'), icon: Play, run: () => act('start') }
    if (view === 'running' || view === 'external') return { label: t('open'), icon: ArrowUpRight, run: () => invoke('open_service') }
    if (view === 'error') return { label: t('retry'), icon: RefreshCw, run: refresh }
    return null
  }, [busy, checking, refresh, t, view])

  const running = view === 'running'
  const workspaceDisabled = running || view === 'starting'
  const runtimeSourceLabel = shownStatus.runtimeSource === 'global'
    ? t('globalRuntime')
    : shownStatus.runtimeSource === 'npx'
      ? t('cachedRuntime')
      : t('managedRuntime')

  const changelogHtml = useMemo(() => {
    if (!changelog?.body) return ''
    return marked.parse(localizedReleaseNotes(changelog.body, locale)) as string
  }, [changelog, locale])
  const publishedDate = formatDate(changelog?.publishedAt ?? null, locale)

  const startWindowDrag = (event: React.MouseEvent<HTMLElement>) => {
    if (event.button !== 0 || !isTauri() || (event.target as HTMLElement).closest('button')) return
    void getCurrentWindow().startDragging()
  }

  return (
    <div className="app-shell">
      <div className="ambient ambient-one" />
      <div className="ambient ambient-two" />

      <header className="titlebar" data-tauri-drag-region onMouseDown={startWindowDrag}>
        <div className="brand" data-tauri-drag-region>
          <img src="/logo.svg" alt="" className="brand-mark" />
          <div data-tauri-drag-region>
            <strong>{t('appName')}</strong>
            <span>{t('tagline')}</span>
          </div>
        </div>
        <div className="toolbar">
          <button className="icon-button" aria-label={t('refresh')} onClick={refresh} disabled={checking || Boolean(busy)}>
            <RefreshCw size={17} className={checking ? 'spin' : ''} />
          </button>
          <button
            className={`icon-button ${preferencesOpen ? 'selected' : ''}`}
            aria-label={t('settings')}
            aria-expanded={preferencesOpen}
            onClick={() => setPreferencesOpen((open) => !open)}
          >
            <Settings2 size={18} />
          </button>
        </div>
      </header>

      {preferencesOpen && (
        <aside className="preferences glass" aria-label={t('settings')}>
          <div className="preference-row">
            <span><Globe2 size={16} />{t('language')}</span>
            <div className="segmented">
              <button className={locale === 'zh-CN' ? 'active' : ''} onClick={() => setLocale('zh-CN')}>中文</button>
              <button className={locale === 'en' ? 'active' : ''} onClick={() => setLocale('en')}>EN</button>
            </div>
          </div>
          <div className="preference-row">
            <span>{theme === 'dark' ? <Moon size={16} /> : <Sun size={16} />}{t('appearance')}</span>
            <div className="segmented">
              {(['system', 'light', 'dark'] as const).map((value) => (
                <button key={value} className={theme === value ? 'active' : ''} onClick={() => setTheme(value)}>{t(value)}</button>
              ))}
            </div>
          </div>
        </aside>
      )}

      <main>
        <section className={`hero glass state-${view}`} aria-live="polite">
          <div className="status-orbit" aria-hidden="true">
            <div className="orbit-glow" />
            <div className="status-core">
              {view === 'checking' || view === 'starting' ? <LoaderCircle className="spin" size={31} /> :
                view === 'error' || view === 'needsNode' ? <CircleAlert size={31} /> :
                  view === 'running' || view === 'external' ? <Check size={32} strokeWidth={2.5} /> : <Play size={29} fill="currentColor" />}
            </div>
          </div>

          <div className="hero-copy">
            <h1>{t(copy.title)}</h1>
            {statusHint && <p>{statusHint}</p>}
          </div>

          <div className="actions">
            {primaryAction && (
              <button className="primary-button" onClick={primaryAction.run}>
                <primaryAction.icon size={18} />
                {busy === 'install' ? t('installing') : busy === 'update' ? t('updating') : busy === 'restart' ? t('restarting') : busy === 'stop' ? t('stopping') : primaryAction.label}
              </button>
            )}
            {running && !busy && (
              <>
                <button className="secondary-button" onClick={() => act('restart')}><RotateCcw size={17} />{t('restart')}</button>
                <button className="secondary-button danger" onClick={() => act('stop')}><Square size={15} fill="currentColor" />{t('stop')}</button>
              </>
            )}
          </div>

          <div className="local-note">{t('localOnly')}</div>
        </section>

        <section className="detail-grid">
          <article className="detail-card glass">
            <div className="detail-heading">
              <div className="detail-icon runtime-icon"><span>DSH</span></div>
              <div><h2>{t('runtime')}</h2><p>{runtimeSourceLabel}</p></div>
              <span className={`small-status ${shownStatus.installed ? 'positive' : ''}`}>
                {shownStatus.installed ? t('installed') : t('missing')}
              </span>
            </div>
            <div className="version-row">
              <span>{t('version')} <strong>{shownStatus.installedVersion ?? '—'}</strong></span>
              <span>{t('latest')} <strong>{shownStatus.latestVersion ?? t('unknown')}</strong></span>
            </div>
            <div className="card-footer">
              <span>{shownStatus.updateAvailable ? t('available') : shownStatus.installed ? t('current') : t('managedRuntimeHint')}</span>
              {shownStatus.updateAvailable && !running && !busy && (
                <button className="text-button" onClick={openChangelog}>{t('update')}<ChevronRight size={14} /></button>
              )}
              {!shownStatus.installed && view === 'external' && !busy && (
                <button className="text-button" onClick={() => act('install')}>{t('install')}<ChevronRight size={14} /></button>
              )}
            </div>
          </article>

          <article className="detail-card glass workspace-card">
            <div className="detail-heading">
              <div className="detail-icon folder-icon"><Folder size={21} /></div>
              <div><h2>{t('workspace')}</h2><p>{t('workspaceHint')}</p></div>
            </div>
            <button className="workspace-picker" onClick={chooseWorkspace} disabled={workspaceDisabled} title={workspaceDisabled ? t('changeWorkspaceWhileRunning') : ''}>
              <span>{shownStatus.workspace || '—'}</span>
              <strong>{t('choose')}</strong>
            </button>
          </article>
        </section>

        <p className="preview-note"><CircleAlert size={14} />{t('developerPreview')}</p>
      </main>

      {changelogOpen && (
        <div className="modal-overlay" onClick={closeChangelog}>
          <div
            className="modal-card glass"
            role="dialog"
            aria-modal="true"
            aria-labelledby="changelog-title"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="modal-header">
              <div className="modal-heading">
                <div className="modal-symbol" aria-hidden="true"><Sparkles size={20} /></div>
                <div>
                  <span className="modal-eyebrow">{t('available')}</span>
                  <h2 id="changelog-title">{t('changelogTitle')}</h2>
                  {changelog && (
                    <p className="modal-meta">
                      <strong>DSH {changelog.version}</strong>
                      {publishedDate && <span>{t('publishedAt')} {publishedDate}</span>}
                    </p>
                  )}
                </div>
              </div>
              <button className="icon-button modal-close" aria-label={t('cancel')} onClick={closeChangelog} disabled={updating}>
                <X size={18} />
              </button>
            </div>

            <div className="modal-body">
              {changelogLoading && (
                <div className="modal-loading">
                  <LoaderCircle className="spin" size={28} />
                  <p>{t('changelogLoading')}</p>
                </div>
              )}
              {changelogError && !changelogLoading && (
                <div className="modal-error">
                  <CircleAlert size={28} />
                  <p>{t('changelogError')}</p>
                  <span className="modal-error-detail">{changelogError}</span>
                </div>
              )}
              {changelog && !changelogLoading && !changelogError && (
                <div
                  className="changelog-content"
                  dangerouslySetInnerHTML={{ __html: changelogHtml }}
                  onClick={handleChangelogClick}
                />
              )}
            </div>

            <div className="modal-footer">
              {changelog?.htmlUrl && !changelogLoading && (
                <button
                  className="text-button"
                  onClick={() => invoke('open_external', { url: changelog.htmlUrl })}
                >
                  {t('viewOnGitHub')}<ArrowUpRight size={13} />
                </button>
              )}
              <div className="modal-footer-actions">
                <button className="secondary-button" onClick={closeChangelog} disabled={updating}>
                  {t('cancel')}
                </button>
                <button
                  className="primary-button"
                  onClick={performUpdate}
                  disabled={changelogLoading || Boolean(changelogError) || updating}
                >
                  {updating ? <LoaderCircle className="spin" size={18} /> : <RefreshCw size={17} />}
                  {updating ? t('updating') : t('updateNow')}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default App
