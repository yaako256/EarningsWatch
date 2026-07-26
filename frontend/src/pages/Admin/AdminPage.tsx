import { Route, Routes, useLocation, useNavigate } from 'react-router-dom'
import { PageHeader } from '../../components/common/PageHeader'
import { AdminDashboardTab } from './AdminDashboardTab'
import { AdminUsersTab } from './AdminUsersTab'
import { AdminNotifyConfigTab } from './AdminNotifyConfigTab'
import { AdminLogsTab } from './AdminLogsTab'

export function AdminPage() {
  const navigate = useNavigate()
  const location = useLocation()

  // タブ切り替えはUI操作のためLinkではなくボタン+navigateで実装する(ホバー時のURL表示を避けるため)。
  const tabs = [
    { to: '/admin', label: 'ダッシュボード', isActive: location.pathname === '/admin' },
    { to: '/admin/users', label: 'ユーザ管理', isActive: location.pathname.startsWith('/admin/users') },
    {
      to: '/admin/system-notifications',
      label: 'システム通知設定',
      isActive: location.pathname.startsWith('/admin/system-notifications'),
    },
    { to: '/admin/logs', label: 'ログ閲覧', isActive: location.pathname.startsWith('/admin/logs') },
  ]

  return (
    <div>
      <PageHeader icon="🛠" title="管理者機能" />
      <div className="tabs">
        {tabs.map((tab) => (
          <button key={tab.to} className={tab.isActive ? 'active' : ''} onClick={() => navigate(tab.to)}>
            {tab.label}
          </button>
        ))}
      </div>

      <Routes>
        <Route index element={<AdminDashboardTab />} />
        <Route path="users" element={<AdminUsersTab />} />
        <Route path="system-notifications" element={<AdminNotifyConfigTab />} />
        <Route path="logs" element={<AdminLogsTab />} />
      </Routes>
    </div>
  )
}
