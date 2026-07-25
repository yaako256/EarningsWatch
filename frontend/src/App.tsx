// アプリ全体のルーティング。設計書7.2節のURLパス構造にそのまま対応させる。

import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { AuthProvider } from './contexts/AuthContext'
import { ToastProvider } from './contexts/ToastContext'
import { RequireAuth, RequireAdmin } from './routes/guards'
import { AppLayout } from './components/layout/AppLayout'
import { LoginPage } from './pages/LoginPage'
import { ForbiddenPage } from './pages/ForbiddenPage'
import { DashboardPage } from './pages/DashboardPage'
import { GroupsPage } from './pages/GroupsPage'
import { GroupDetailLayout, GroupDetailIndexRedirect } from './pages/Groups/GroupDetailLayout'
import { GroupSettingsTab } from './pages/Groups/GroupSettingsTab'
import { GroupFiltersTab } from './pages/Groups/GroupFiltersTab'
import { EarningsPage } from './pages/EarningsPage'
import { EarningsSummaryPage } from './pages/EarningsSummaryPage'
import { DeliveriesPage } from './pages/DeliveriesPage'
import { AnnouncementsPage } from './pages/AnnouncementsPage'
import { AdminPage } from './pages/Admin/AdminPage'

export default function App() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <ToastProvider>
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/forbidden" element={<ForbiddenPage />} />

            <Route element={<RequireAuth />}>
              <Route element={<AppLayout />}>
                <Route path="/" element={<Navigate to="/dashboard" replace />} />
                <Route path="/dashboard" element={<DashboardPage />} />

                <Route path="/groups" element={<GroupsPage />} />
                <Route path="/groups/:groupId" element={<GroupDetailLayout />}>
                  <Route index element={<GroupDetailIndexRedirect />} />
                  <Route path="filters" element={<GroupFiltersTab />} />
                  <Route path="settings" element={<GroupSettingsTab />} />
                </Route>

                <Route path="/earnings" element={<EarningsPage />} />
                <Route path="/earnings/summary" element={<EarningsSummaryPage />} />

                <Route path="/deliveries/*" element={<DeliveriesPage />} />
                <Route path="/announcements/*" element={<AnnouncementsPage />} />

                <Route element={<RequireAdmin />}>
                  <Route path="/admin/*" element={<AdminPage />} />
                </Route>
              </Route>
            </Route>

            <Route path="*" element={<Navigate to="/dashboard" replace />} />
          </Routes>
        </ToastProvider>
      </AuthProvider>
    </BrowserRouter>
  )
}
