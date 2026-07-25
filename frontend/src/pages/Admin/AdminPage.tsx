import { NavLink, Route, Routes } from 'react-router-dom'
import { PageHeader } from '../../components/common/PageHeader'
import { AdminDashboardTab } from './AdminDashboardTab'
import { AdminUsersTab } from './AdminUsersTab'
import { AdminNotifyConfigTab } from './AdminNotifyConfigTab'
import { AdminLogsTab } from './AdminLogsTab'

export function AdminPage() {
  return (
    <div>
      <PageHeader icon="🛠" title="管理者機能" />
      <div className="tabs">
        <NavLink to="/admin" end>
          {({ isActive }) => <button className={isActive ? 'active' : ''}>ダッシュボード</button>}
        </NavLink>
        <NavLink to="/admin/users">
          {({ isActive }) => <button className={isActive ? 'active' : ''}>ユーザ管理</button>}
        </NavLink>
        <NavLink to="/admin/system-notifications">
          {({ isActive }) => <button className={isActive ? 'active' : ''}>システム通知設定</button>}
        </NavLink>
        <NavLink to="/admin/logs">
          {({ isActive }) => <button className={isActive ? 'active' : ''}>ログ閲覧</button>}
        </NavLink>
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
