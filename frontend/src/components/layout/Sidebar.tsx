// サイドバー。設計書11.8節に準拠:
//   - 折りたたみ時もアイコンのみ常時表示、展開時は文字ラベルも表示
//   - 最上部にアプリ名+アイコン、最下部にユーザーアイコン+ユーザー名+ログアウト
//
// 改善点まとめ資料への対応: 折りたたみ時、アイコンがサイドバー幅の中央に収まるよう
// アイテムをフレックスで中央揃えにする(以前はアイコンが左に寄っていた)。

import { NavLink } from 'react-router-dom'
import { useState } from 'react'
import { useAuth } from '../../contexts/AuthContext'

interface NavItem {
  to: string
  icon: string
  label: string
}

const NAV_ITEMS: NavItem[] = [
  { to: '/dashboard', icon: '📊', label: 'ダッシュボード' },
  { to: '/groups', icon: '👥', label: 'グループ管理' },
  { to: '/earnings', icon: '📈', label: '決算情報' },
  { to: '/deliveries', icon: '📬', label: '送信キュー/履歴' },
  { to: '/announcements', icon: '📌', label: 'お知らせ板' },
]

const ADMIN_NAV_ITEM: NavItem = { to: '/admin', icon: '🛠', label: '管理者' }

export function Sidebar() {
  const [collapsed, setCollapsed] = useState(false)
  const { user, logout } = useAuth()

  const items = user?.role === 'admin' ? [...NAV_ITEMS, ADMIN_NAV_ITEM] : NAV_ITEMS
  const initial = user?.username ? user.username.charAt(0).toUpperCase() : '?'

  return (
    <aside className={`sidebar ${collapsed ? 'collapsed' : ''}`}>
      <div className="sidebar-top">
        <span className="sidebar-logo-icon">📡</span>
        {!collapsed && <span className="sidebar-logo-text">EarningsWatch</span>}
        <button
          className="sidebar-collapse-toggle"
          onClick={() => setCollapsed((prev) => !prev)}
          aria-label={collapsed ? 'サイドバーを開く' : 'サイドバーを閉じる'}
          title={collapsed ? 'サイドバーを開く' : 'サイドバーを閉じる'}
        >
          {collapsed ? '»' : '«'}
        </button>
      </div>

      <nav className="sidebar-nav">
        {items.map((item) => (
          <NavLink
            key={item.to}
            to={item.to}
            className={({ isActive }) => `sidebar-item ${isActive ? 'active' : ''}`}
            title={collapsed ? item.label : undefined}
          >
            <span className="sidebar-item-icon">{item.icon}</span>
            {!collapsed && <span className="sidebar-item-label">{item.label}</span>}
          </NavLink>
        ))}
      </nav>

      <div className="sidebar-bottom">
        <div className="sidebar-user" title={collapsed ? user?.username : undefined}>
          <span className="user-avatar">{initial}</span>
          {!collapsed && <span className="sidebar-user-name">{user?.username}</span>}
        </div>
        <button
          className="sidebar-logout"
          onClick={() => void logout()}
          title="ログアウト"
        >
          <span className="sidebar-item-icon">🚪</span>
          {!collapsed && <span>ログアウト</span>}
        </button>
      </div>
    </aside>
  )
}
