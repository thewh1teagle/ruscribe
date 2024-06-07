import '@fontsource/roboto'
import { event, path } from '@tauri-apps/api'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import * as webview from '@tauri-apps/api/webviewWindow'
import { onOpenUrl } from '@tauri-apps/plugin-deep-link'
import * as dialog from '@tauri-apps/plugin-dialog'
import * as fs from '@tauri-apps/plugin-fs'
import * as os from '@tauri-apps/plugin-os'
import { open } from '@tauri-apps/plugin-shell'
import { useContext, useEffect, useRef, useState } from 'react'
import { useLocation, useNavigate } from 'react-router-dom'
import successSound from '~/assets/success.mp3'
import { TextFormat } from '~/components/FormatSelect'
import * as config from '~/lib/config'
import * as transcript from '~/lib/transcript'
import { NamedPath, ls, openPath, pathToNamedPath } from '~/lib/utils'
import { getX86Features } from '~/lib/x86Features'
import { ErrorModalContext } from '~/providers/ErrorModal'
import { ModelOptions, usePreferencesContext } from '~/providers/Preferences'
import { UpdaterContext } from '~/providers/Updater'

export interface BatchOptions {
	files: NamedPath[]
	format: TextFormat
	modelOptions: ModelOptions
}

export function viewModel() {
	const location = useLocation()
	const [settingsVisible, setSettingsVisible] = useState(location.hash === '#settings')
	const navigate = useNavigate()
	const [loading, setLoading] = useState(false)
	const abortRef = useRef<boolean>(false)
	const [isAborting, setIsAborting] = useState(false)
	const [segments, setSegments] = useState<transcript.Segment[] | null>(null)
	const [audio, setAudio] = useState<HTMLAudioElement | null>(null)
	const [progress, setProgress] = useState<number | null>(0)
	const [files, setFiles] = useState<NamedPath[]>(location?.state?.files ?? [])
	const preferences = usePreferencesContext()

	const { updateApp, availableUpdate } = useContext(UpdaterContext)
	const { setState: setErrorModal } = useContext(ErrorModalContext)

	async function onFilesChanged() {
		if (files.length === 1) {
			setAudio(new Audio(convertFileSrc(files[0].path)))
		}
	}
	useEffect(() => {
		onFilesChanged()
	}, [files])

	async function handleNewSegment() {
		await listen('transcribe_progress', (event) => {
			const value = event.payload as number
			if (value >= 0 && value <= 100) {
				setProgress(value)
			}
		})
		await listen<transcript.Segment>('new_segment', (event) => {
			const { payload } = event
			setSegments((prev) => (prev ? [...prev, payload] : [payload]))
		})
	}

	async function onAbort() {
		setIsAborting(true)
		abortRef.current = true
		event.emit('abort_transcribe')
	}

	async function selectFiles() {
		const selected = await dialog.open({
			multiple: true,
			filters: [
				{
					name: 'Audio or Video files',
					extensions: [...config.audioExtensions, ...config.videoExtensions],
				},
			],
		})
		if (selected) {
			const newFiles: NamedPath[] = []
			for (const file of selected) {
				newFiles.push({ name: file.name ?? '', path: file.path })
			}
			setFiles(newFiles)

			if (newFiles.length > 1) {
				navigate('/batch', { state: { files: newFiles } })
			}
		}
	}

	async function checkModelExists() {
		try {
			const configPath = await path.appLocalDataDir()
			const entries = await ls(configPath)
			const filtered = entries.filter((e) => e.name?.endsWith('.bin'))
			if (filtered.length === 0) {
				// Download new model if no models and it's not manual installation
				if (!preferences.skippedSetup) {
					navigate('/setup')
				}
			} else {
				if (!preferences.modelPath || !(await fs.exists(preferences.modelPath))) {
					// if model path not found set another one as default
					const absPath = await path.join(configPath, filtered[0].name)
					preferences.setModelPath(absPath)
				}
			}
		} catch (e) {
			console.error(e)
			navigate('/setup')
		}
	}

	async function handleDrop() {
		listen<{ paths: string[] }>('tauri://drop', async (event) => {
			const newFiles: NamedPath[] = []
			for (const path of event.payload.paths) {
				const file = await pathToNamedPath(path)
				newFiles.push({ name: file.name, path: file.path })
			}
			setFiles(newFiles)
			if (newFiles.length > 1) {
				navigate('/batch', { state: { files: newFiles } })
			}
		})
	}

	async function handleDeepLinks() {
		const platform = await os.platform()
		const newFiles = []
		if (platform === 'macos') {
			await onOpenUrl(async (urls) => {
				for (let url of urls) {
					if (url.startsWith('file://')) {
						url = decodeURIComponent(url)
						url = url.replace('file://', '')
						// take only the first one
						newFiles.push(await pathToNamedPath(url))
					}
				}
			})
		} else if (platform == 'windows' || platform == 'linux') {
			const urls: string[] = await invoke('get_deeplinks')
			for (const url of urls) {
				newFiles.push(await pathToNamedPath(url))
			}
		}
		setFiles([...files, ...newFiles])
		if (newFiles.length > 1) {
			navigate('/batch', { state: { files: newFiles } })
		}
	}

	async function CheckCpuAndInit() {
		const features = await getX86Features()
		if (features) {
			const unsupported = Object.entries(features || {})
				.filter(([_, feature]) => feature.enabled && !feature.support)
				.map(([name]) => name)
			if (unsupported.length > 0) {
				// Found unsupported features
				await dialog.message(
					`Your CPU is old and doesn't support some features (${unsupported.join(
						','
					)}). Please click OK and read the readme that will open for more information.`,
					{
						kind: 'error',
					}
				)
				open(config.unsupportedCpuReadmeURL)
				return // Don't run anything
			}
		}

		handleDrop()
		handleDeepLinks()
		checkModelExists()
		handleNewSegment()
	}

	useEffect(() => {
		CheckCpuAndInit()
	}, [])

	async function transcribe() {
		setSegments(null)
		setLoading(true)
		try {
			const options = {
				path: files[0].path,
				model_path: preferences.modelPath,
				...preferences.modelOptions,
			}
			const startTime = performance.now()
			const res: transcript.Transcript = await invoke('transcribe', { options })

			// Calcualte time
			const total = Math.round((performance.now() - startTime) / 1000)
			console.info(`Transcribe took ${total} seconds.`)

			setSegments(res.segments)
		} catch (error) {
			if (!abortRef.current) {
				console.error('error: ', error)
				setErrorModal?.({ log: String(error), open: true })
				setLoading(false)
			}
		} finally {
			setLoading(false)
			setIsAborting(false)
			setProgress(null)
			if (!abortRef.current) {
				// Focus back the window and play sound
				if (preferences.soundOnFinish) {
					new Audio(successSound).play()
				}
				if (preferences.focusOnFinish) {
					webview.getCurrent().unminimize()
					webview.getCurrent().setFocus()
				}
			}
		}
	}

	return {
		preferences,
		openPath,
		selectFiles,
		isAborting,
		settingsVisible,
		setSettingsVisible,
		loading,
		progress,
		audio,
		setAudio,
		files,
		setFiles,
		availableUpdate,
		updateApp,
		segments,
		transcribe,
		onAbort,
	}
}
