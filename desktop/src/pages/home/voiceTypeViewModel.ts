import { invoke } from '@tauri-apps/api/core'
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut'
import { useEffect, useState } from 'react'
import { usePreferenceProvider } from '~/providers/Preference'

export function viewModel() {
	const [active, setActive] = useState(false)
	const preference = usePreferenceProvider()

	async function setupKeyboardShortcut() {
		await unregisterAll()
		await register('Ctrl+J', (event) => {
			setActive(event.state == 'Pressed')
		})
	}

	async function loadModel() {
		await invoke('load_model', { modelPath: preference.modelPath, gpuDevice: preference.gpuDevice })
	}
	useEffect(() => {
		setupKeyboardShortcut()
		loadModel()
	}, [])



	return {
		active,
	}
}
