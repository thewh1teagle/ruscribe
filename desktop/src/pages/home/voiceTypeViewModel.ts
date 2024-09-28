import { unregister } from '@tauri-apps/plugin-deep-link'
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut'
import { useEffect, useState } from 'react'

export function viewModel() {
	const [active, setActive] = useState(false)

	async function setupKeyboardShortcut() {
		await unregisterAll()
		await register('Ctrl+J', (event) => {
			setActive(event.state == 'Pressed')
		})
	}
	useEffect(() => {
		setupKeyboardShortcut()
	}, [])

	return {
		active,
	}
}
