import { useTranslation } from 'react-i18next'
import FormatSelect from '~/components/FormatSelect'
import LanguageInput from '~/components/LanguageInput'
import Layout from '~/components/Layout'
import ModelOptions from '~/components/Params'
import BatchPanel from './BatchPanel'
import { viewModel } from './viewModel'
import { cx } from '~/lib/utils'

export default function BatchPage() {
	const vm = viewModel()
	const { t } = useTranslation()
	return (
		<Layout>
			<div className="m-auto w-[80%] max-w-[300px]">
				<LanguageInput lang={vm.lang} setLang={vm.setLang} />

				<FormatSelect setFormat={vm.setFormat} format={vm.format} />
				<ModelOptions args={vm.args} setArgs={vm.setArgs} />

				<div className="mt-5">
					<BatchPanel index={vm.index} inProgress={vm.inProgress} progress={vm.progress} onCancel={vm.cancel} onStart={vm.start} files={vm.files} />
					{!vm.inProgress && (
						<div onMouseDown={vm.selectFiles} className={cx('text-xs text-base-content font-medium cursor-pointer ms-2 mt-1.5')}>
							{t('common.change-files')}
						</div>
					)}
				</div>
			</div>
		</Layout>
	)
}
