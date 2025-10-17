import { BrowserRouter as Router, Routes, Route, Link, useLocation } from 'react-router-dom'
import CameraManagementPage from './pages/CameraManagementPage'
import ConfigEditorPage from './pages/ConfigEditorPage'
import ConfigDeployPage from './pages/ConfigDeployPage'
import LogsPage from './pages/LogsPage'
import SystemStatus from './components/SystemStatus'

function App() {
  return (
    <Router>
      <AppContent />
    </Router>
  )
}

function AppContent() {
  const location = useLocation()

  const navItems = [
    { path: '/', label: '主页', icon: '🏠' },
    { path: '/config-deploy', label: '配置与部署', icon: '🚀' },
    { path: '/cameras', label: '摄像头管理', icon: '📷' },
    { path: '/config-editor', label: '配置编辑器', icon: '📝' },
    { path: '/logs', label: '日志', icon: '📋' },
  ]

  return (
    <div className="min-h-screen bg-gray-50">
      <nav className="bg-white shadow-sm">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between h-16">
            <div className="flex">
              <div className="flex-shrink-0 flex items-center">
                <h1 className="text-xl font-bold text-blue-600">
                  Frigate 配置工具
                </h1>
              </div>
              <div className="hidden sm:ml-6 sm:flex sm:space-x-4">
                {navItems.map(item => (
                  <Link
                    key={item.path}
                    to={item.path}
                    className={`${
                      location.pathname === item.path
                        ? 'border-blue-500 text-gray-900'
                        : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
                    } inline-flex items-center px-3 pt-1 border-b-2 text-sm font-medium`}
                  >
                    <span className="mr-2">{item.icon}</span>
                    {item.label}
                  </Link>
                ))}
              </div>
            </div>
            {/* Phase 4 - 系统状态指示器 */}
            <div className="hidden lg:flex items-center">
              <SystemStatus />
            </div>
          </div>
        </div>
      </nav>

      <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/config-deploy" element={<ConfigDeployPage />} />
          <Route path="/cameras" element={<CameraManagementPage />} />
          <Route path="/config-editor" element={<ConfigEditorPage />} />
          <Route path="/logs" element={<LogsPage />} />
        </Routes>
      </main>
    </div>
  )
}

function HomePage() {
  return (
    <div className="px-4 py-6 sm:px-0">
      <div className="bg-white shadow rounded-lg p-8">
        <h2 className="text-3xl font-bold text-gray-900 mb-4">
          欢迎使用 Frigate 配置工具
        </h2>
        <p className="text-lg text-gray-600 mb-6">
          这是一个跨平台的可视化配置工具，用于简化 Frigate NVR 的设置过程。
        </p>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-6 mt-8">
          <FeatureCard
            icon="🔧"
            title="硬件检测"
            description="自动检测 GPU、TPU 和视频设备"
            status="Phase 3"
          />
          <FeatureCard
            icon="🔍"
            title="摄像头发现"
            description="扫描内网并自动发现 IP 摄像头"
            status="Phase 8.5"
          />
          <FeatureCard
            icon="📷"
            title="相机配置"
            description="可视化配置相机并分配硬件加速"
            status="Phase 3"
          />
          <FeatureCard
            icon="📝"
            title="配置编辑器"
            description="YAML编辑、验证和快照管理"
            status="Phase 4"
          />
          <FeatureCard
            icon="🚀"
            title="安全部署"
            description="预验证和健康检查"
            status="Phase 5"
          />
          <FeatureCard
            icon="💾"
            title="磁盘映射"
            description="配置录像和片段存储位置"
            status="Phase 8"
          />
          <FeatureCard
            icon="📋"
            title="日志查看"
            description="实时流式日志监控"
            status="Phase 5"
          />
        </div>

        <div className="mt-8 p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <p className="text-sm text-blue-800">
            <strong>当前状态:</strong> Phase 2 (基础设施) 进行中。核心功能将在 Phase 3-8 中实现。
          </p>
        </div>
      </div>
    </div>
  )
}

function FeatureCard({ icon, title, description, status }: {
  icon: string
  title: string
  description: string
  status: string
}) {
  return (
    <div className="bg-gray-50 rounded-lg p-6 border border-gray-200">
      <div className="text-3xl mb-3">{icon}</div>
      <h3 className="text-lg font-semibold text-gray-900 mb-2">{title}</h3>
      <p className="text-sm text-gray-600 mb-3">{description}</p>
      <span className="inline-block px-3 py-1 text-xs font-medium text-gray-700 bg-gray-200 rounded-full">
        {status}
      </span>
    </div>
  )
}

// Placeholder pages - reserved for future phases
// function PlaceholderPage({ title, phase }: { title: string; phase: string }) {
//   return (
//     <div className="px-4 py-6 sm:px-0">
//       <div className="bg-white shadow rounded-lg p-8 text-center">
//         <h2 className="text-2xl font-semibold text-gray-900 mb-4">{title}</h2>
//         <p className="text-gray-600 mb-4">此页面将在 {phase} 中实现</p>
//         <div className="text-6xl mb-4">🚧</div>
//         <p className="text-sm text-gray-500">请稍后查看</p>
//       </div>
//     </div>
//   )
// }

export default App
