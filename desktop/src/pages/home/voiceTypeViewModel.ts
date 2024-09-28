import { unregister } from '@tauri-apps/plugin-deep-link'
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut'
import { useEffect } from 'react'

export function viewModel() {
	async function setupKeyboardShortcut() {
		await unregisterAll()
		await register('Ctrl+J', (event) => {
			console.log('Shortcut triggered', event.state)
		})
	}
	useEffect(() => {
		setupKeyboardShortcut()
	}, [])
}
