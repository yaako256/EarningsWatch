// discord/slackの通知先設定フォーム。グループ詳細の設定タブと、管理者のシステム通知設定画面で共有する。

import { MaskedField } from './MaskedField'
import { EmbedColorPicker } from './EmbedColorPicker'
import { MentionTargetsEditor } from './MentionTargetsEditor'
import type { GroupConfig } from '../../types/api'

interface GroupConfigFormProps {
  config: GroupConfig
  onChange: (config: GroupConfig) => void
}

export function GroupConfigForm({ config, onChange }: GroupConfigFormProps) {
  return (
    <div className="settings-form">
      <label>
        Webhook URL
        <MaskedField
          value={config.webhookUrl ?? ''}
          onChange={(webhookUrl) => onChange({ ...config, webhookUrl: webhookUrl || null })}
          placeholder="https://..."
        />
      </label>

      {config.medium === 'discord' && (
        <label>
          Embedの色
          <EmbedColorPicker
            value={config.embedColor}
            onChange={(embedColor) => onChange({ ...config, embedColor })}
          />
        </label>
      )}

      <label className="check">
        <input
          type="checkbox"
          checked={config.mentionEnabled}
          onChange={(e) => onChange({ ...config, mentionEnabled: e.target.checked })}
        />
        メンションを有効にする
      </label>

      {config.mentionEnabled && (
        <label>
          メンション対象
          <MentionTargetsEditor
            targets={config.mentionTargets}
            onChange={(mentionTargets) => onChange({ ...config, mentionTargets })}
          />
        </label>
      )}
    </div>
  )
}
