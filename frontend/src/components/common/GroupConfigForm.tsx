// discord/slackの通知先設定フォーム。グループ詳細の設定タブと、管理者のシステム通知設定画面で共有する。
//
// Ver2改善点対応: 「後々、グループ作成後にも送信媒体を変更できるようにしたい」との要望のため、
// 媒体変更セレクトは常時表示しておく(Slackは現時点で開発中のため選択時に警告のみ表示する)。

import { MaskedField } from './MaskedField'
import { EmbedColorPicker } from './EmbedColorPicker'
import { MentionTargetsEditor } from './MentionTargetsEditor'
import type { GroupConfig } from '../../types/api'

interface GroupConfigFormProps {
  config: GroupConfig
  onChange: (config: GroupConfig) => void
}

function switchMedium(config: GroupConfig, medium: 'discord' | 'slack'): GroupConfig {
  if (medium === config.medium) return config
  if (medium === 'discord') {
    return {
      medium: 'discord',
      webhookUrl: config.webhookUrl,
      embedColor: null,
      mentionEnabled: config.mentionEnabled,
      mentionTargets: config.mentionTargets,
    }
  }
  return {
    medium: 'slack',
    webhookUrl: config.webhookUrl,
    mentionEnabled: config.mentionEnabled,
    mentionTargets: config.mentionTargets,
  }
}

export function GroupConfigForm({ config, onChange }: GroupConfigFormProps) {
  return (
    <div className="settings-form">
      <label>
        通知媒体
        <select
          value={config.medium}
          onChange={(e) => onChange(switchMedium(config, e.target.value as 'discord' | 'slack'))}
        >
          <option value="discord">Discord</option>
          <option value="slack">Slack(開発中)</option>
        </select>
      </label>
      {config.medium === 'slack' && (
        <p className="inline-error">
          Slackは現時点で開発中の仮実装のため、実質的にご利用いただけません。
        </p>
      )}

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

      {/* Ver3改善点対応: <label>でinputとテキストを丸ごと囲むと、テキスト部分をクリックしても
          反応してしまい「判定が広すぎる」という指摘につながった。inputだけを独立させ、
          テキストは装飾のspanにしてクリックしても何も起きないようにする。 */}
      <div className="check-row">
        <input
          id="mention-enabled"
          type="checkbox"
          checked={config.mentionEnabled}
          onChange={(e) => onChange({ ...config, mentionEnabled: e.target.checked })}
        />
        <span>メンションを有効にする</span>
      </div>

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
