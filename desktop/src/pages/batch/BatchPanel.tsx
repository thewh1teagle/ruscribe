import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ReactComponent as CancelIcon } from '~/icons/cancel.svg'
import { ReactComponent as ChevronUp } from '~/icons/chevron-down.svg'
import { ReactComponent as ChevronDown } from '~/icons/chevron-up.svg'
import { ReactComponent as PlayIcon } from '~/icons/play.svg'
import { NamedPath, cx } from '~/lib/utils'
import BatchQueue from './BatchQueue'

interface BatchPanelProps {
	files: NamedPath[]
	onStart: () => void
	onCancel: () => void
	progress: number | null
	index: number
	inProgress: boolean
}

export default function BatchPanel({ files, onStart, onCancel, progress, index, inProgress }: BatchPanelProps) {
	const { t } = useTranslation()
	const [open, setOpen] = useState(true)

	return (
		<div className={cx('collapse border border-base-300 bg-base-200', open && 'collapse-open')}>
			<div className="collapse-title !cursor-default text-xl font-medium flex p-3 flex-row items-center">
				<div className="flex flex-col ms-4">
					{inProgress && (
						<span className="text-sm font-light tracking-wider">
							{t('common.transcribing')} ({index + 1}/{files.length})
						</span>
					)}
					{!inProgress && (
						<span className="text-sm font-light tracking-wider">
							{t('common.transcribe')} {files.length} {t('common.files')}
						</span>
					)}
				</div>
				<div className="ms-auto flex gap-3">
					<div className="cursor-pointer p-2 bg-neutral rounded-full" onMouseDown={() => setOpen(!open)}>
						{open ? <ChevronDown className="h-5 w-5" /> : <ChevronUp className="h-5 w-5" />}
					</div>
					<div className={cx('cursor-pointer p-2 rounded-full', inProgress && 'bg-error', !inProgress && 'bg-success')}>
						{inProgress ? <CancelIcon onClick={onCancel} className="h-5 w-5" /> : <PlayIcon onClick={onStart} className="h-5 w-5" />}
					</div>
				</div>
			</div>
			<div className="collapse-content">
				<BatchQueue progress={progress} index={index} files={files} />
			</div>
		</div>
	)
}
