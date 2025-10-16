# 实施任务清单: 修复关键 Bug 并优化系统工作流程

**提案 ID**: fix-critical-bugs-and-workflow
**执行模式**: 🚀 全自动连续开发，不停顿询问

## 开发原则

1. 🇨🇳 **全程中文** - 所有注释、日志、提交信息使用中文
2. 🚀 **连续执行** - 一次性完成所有任务，不停顿
3. ✅ **只修复不添加** - 不添加新功能，只修复现有 Bug
4. ✅ **自动化** - 自动测试、格式化、提交
5. ✅ **测试驱动** - 每个修复必须通过测试验证

---

## Phase 1: 摄像头发现页修复 (T001-T010)

### T001: 添加网络接口检测命令 ✅ Backend
**文件**: `src-tauri/src/commands/camera.rs`
```rust
/// 获取本机网络接口信息
#[tauri::command]
pub async fn get_network_interfaces() -> Result<Vec<NetworkInterface>, AppError> {
    // 返回所有网络接口的 IP 和子网信息
    // 如: [{name: "eth0", ip: "192.168.1.100", subnet: "192.168.1.0/24"}]
}
```

### T002: 实现网络接口扫描逻辑 ✅ Backend
**文件**: `src-tauri/src/network/camera_discovery.rs`
```rust
/// 获取本机所有网络接口
pub fn scan_network_interfaces() -> Vec<NetworkInterface> {
    // 使用 system 命令或网络库获取接口信息
    // 过滤掉 loopback 和虚拟接口
}

/// 智能推测扫描范围
pub fn guess_scan_range(interface: &NetworkInterface) -> String {
    // 根据接口 IP 推测网段
    // 如 192.168.1.100 -> 192.168.1.1-254
}
```

### T003: 添加扫描结果持久化 ✅ Frontend
**文件**: `src-ui/src/pages/CameraDiscoveryPage.tsx`
```typescript
// 使用 localStorage 保存扫描结果
const SCAN_RESULTS_KEY = 'camera_scan_results'
const SCAN_TIMESTAMP_KEY = 'camera_scan_timestamp'

// 保存扫描结果
const saveScanResults = (results: Camera[], timestamp: Date) => {
    localStorage.setItem(SCAN_RESULTS_KEY, JSON.stringify(results))
    localStorage.setItem(SCAN_TIMESTAMP_KEY, timestamp.toISOString())
}

// 加载保存的结果
const loadScanResults = () => {
    const saved = localStorage.getItem(SCAN_RESULTS_KEY)
    const timestamp = localStorage.getItem(SCAN_TIMESTAMP_KEY)
    if (saved && timestamp) {
        return {
            results: JSON.parse(saved),
            timestamp: new Date(timestamp)
        }
    }
    return null
}
```

### T004: 修改页面加载逻辑 ✅ Frontend
**文件**: `src-ui/src/pages/CameraDiscoveryPage.tsx`
```typescript
useEffect(() => {
    // 1. 获取本机网络接口
    const loadNetworkInfo = async () => {
        const interfaces = await getNetworkInterfaces()
        setLocalNetwork(interfaces[0]) // 使用第一个有效接口

        // 2. 自动设置扫描范围
        const range = await guessScanRange(interfaces[0])
        setIpRange(range)
    }

    // 3. 恢复上次扫描结果
    const saved = loadScanResults()
    if (saved) {
        setDiscoveredCameras(saved.results)
        setLastScanTime(saved.timestamp)
    }

    loadNetworkInfo()
}, [])
```

### T005: 添加清除结果功能 ✅ Frontend
**文件**: `src-ui/src/pages/CameraDiscoveryPage.tsx`
```typescript
const handleClearResults = () => {
    if (confirm('确定要清除所有扫描结果吗？')) {
        localStorage.removeItem(SCAN_RESULTS_KEY)
        localStorage.removeItem(SCAN_TIMESTAMP_KEY)
        setDiscoveredCameras([])
        setLastScanTime(null)
    }
}
```

---

## Phase 2: 配置验证服务强化 (T011-T020)

### T011: 实现完整的 YAML 验证 ✅ Backend
**文件**: `src-tauri/src/config_engine/validator.rs`
```rust
/// 验证 Frigate 配置的完整性和正确性
pub fn validate_frigate_config(yaml_content: &str) -> ValidationResult {
    let mut errors = Vec::new()
    let mut warnings = Vec::new()

    // 1. 解析 YAML
    let config = match serde_yaml::from_str::<FrigateConfig>(yaml_content) {
        Ok(c) => c,
        Err(e) => {
            errors.push(ValidationError {
                level: "error",
                message: format!("YAML 语法错误: {}", e),
                line: e.location().map(|l| l.line()),
                suggestion: "检查缩进和语法格式"
            })
            return ValidationResult { valid: false, errors, warnings }
        }
    }

    // 2. 检查必需字段
    if config.mqtt.is_none() {
        errors.push(ValidationError {
            level: "error",
            message: "缺少 mqtt 配置",
            line: None,
            suggestion: "添加 mqtt 配置节"
        })
    }

    // 3. 验证摄像头配置
    for (name, camera) in &config.cameras {
        // 检查输入源
        if !camera.ffmpeg.inputs.iter().any(|i| i.path.starts_with("rtsp://")) {
            warnings.push(ValidationWarning {
                level: "warning",
                message: format!("摄像头 {} 没有 RTSP 输入源", name),
                suggestion: "添加 rtsp:// 格式的输入源"
            })
        }
    }

    // 4. 验证检测器配置
    if let Some(detectors) = &config.detectors {
        // 验证检测器类型和设备路径
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings
    }
}
```

### T012: 添加验证命令 ✅ Backend
**文件**: `src-tauri/src/commands/config.rs`
```rust
#[tauri::command]
pub async fn validate_config_content(
    content: String
) -> Result<ValidationResponse, AppError> {
    let result = validate_frigate_config(&content)

    Ok(ValidationResponse {
        valid: result.valid,
        errors: result.errors,
        warnings: result.warnings,
        fixable: check_auto_fixable(&result.errors)
    })
}
```

### T013: 创建前端验证服务 ✅ Frontend
**文件**: `src-ui/src/services/configValidator.ts`
```typescript
export interface ValidationResult {
    valid: boolean
    errors: ValidationError[]
    warnings: ValidationWarning[]
    fixable: boolean
}

export class ConfigValidator {
    // 前端基础验证
    static validateYaml(content: string): ValidationResult {
        const errors: ValidationError[] = []
        const warnings: ValidationWarning[] = []

        // 1. 检查基本 YAML 格式
        try {
            yaml.load(content)
        } catch (e) {
            errors.push({
                level: 'error',
                message: `YAML 解析错误: ${e.message}`,
                line: e.mark?.line,
                suggestion: '检查缩进是否正确（使用空格而非 Tab）'
            })
        }

        // 2. 检查必需的顶级键
        const requiredKeys = ['mqtt', 'cameras', 'detect']
        const doc = yaml.load(content) as any

        for (const key of requiredKeys) {
            if (!doc || !doc[key]) {
                errors.push({
                    level: 'error',
                    message: `缺少必需的配置节: ${key}`,
                    suggestion: `添加 ${key}: 配置`
                })
            }
        }

        return {
            valid: errors.length === 0,
            errors,
            warnings,
            fixable: this.canAutoFix(errors)
        }
    }

    // 调用后端深度验证
    static async validateWithBackend(content: string): Promise<ValidationResult> {
        return await invoke('validate_config_content', { content })
    }
}
```

### T014: 修改配置编辑器页面 ✅ Frontend
**文件**: `src-ui/src/pages/ConfigEditorPage.tsx`
```typescript
// 替换原有的简单验证
const handleValidate = async () => {
    setValidating(true)
    setValidationErrors([])

    try {
        // 1. 前端快速验证
        const quickResult = ConfigValidator.validateYaml(configContent)
        if (!quickResult.valid) {
            setValidationErrors(quickResult.errors)
            setValidationWarnings(quickResult.warnings)
            setCanAutoFix(quickResult.fixable)
            return
        }

        // 2. 后端深度验证
        const deepResult = await ConfigValidator.validateWithBackend(configContent)
        setValidationErrors(deepResult.errors)
        setValidationWarnings(deepResult.warnings)
        setCanAutoFix(deepResult.fixable)

        if (deepResult.valid) {
            toast.success('✅ 配置验证通过')
        } else {
            toast.error(`❌ 发现 ${deepResult.errors.length} 个错误`)
        }
    } finally {
        setValidating(false)
    }
}
```

### T015: 添加错误定位和高亮 ✅ Frontend
**文件**: `src-ui/src/pages/ConfigEditorPage.tsx`
```typescript
// 在编辑器中高亮错误行
const highlightErrors = (errors: ValidationError[]) => {
    errors.forEach(error => {
        if (error.line) {
            editorRef.current?.addLineHighlight(error.line, 'error')
        }
    })
}

// 显示错误详情面板
const ErrorPanel = ({ errors, warnings }) => (
    <div className="validation-panel">
        {errors.map((error, idx) => (
            <div key={idx} className="error-item" onClick={() => goToLine(error.line)}>
                <span className="error-icon">❌</span>
                <div>
                    <p className="error-message">{error.message}</p>
                    {error.suggestion && (
                        <p className="error-suggestion">💡 {error.suggestion}</p>
                    )}
                </div>
            </div>
        ))}
    </div>
)
```

---

## Phase 3: 部署页集成修复 (T021-T030)

### T021: 修复部署页硬件设备加载 ✅ Backend
**文件**: `src-tauri/src/commands/deploy.rs`
```rust
// 确保 deploy_frigate 命令读取保存的硬件设备
pub async fn deploy_frigate(
    request: DeploymentRequestInput,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<DeploymentResponse, AppError> {
    info!("开始部署 Frigate")

    // 1. 加载已保存的硬件设备
    let saved_devices = load_hardware_devices().await?
    info!("加载了 {} 个硬件设备", saved_devices.len())

    // 2. 合并到请求中
    let mut all_devices = request.devices.clone()
    for device in saved_devices {
        if !all_devices.contains(&device.device_path) {
            all_devices.push(device.device_path)
        }
    }

    // 3. 更新 docker-compose.yml
    update_docker_compose_for_deployment(&all_devices).await?

    // 继续部署流程...
}
```

### T022: 统一 docker-compose.yml 路径管理 ✅ Backend
**文件**: `src-tauri/src/commands/deploy.rs`
```rust
// 使用与硬件页面相同的路径查找逻辑
async fn get_deployment_compose_path() -> Result<String, AppError> {
    // 1. 检查用户自定义路径
    if let Ok(Some(custom)) = get_docker_compose_path_internal().await {
        return Ok(custom)
    }

    // 2. 使用标准路径查找
    find_docker_compose_file().await
}
```

### T023: 修复卷映射页面路由 ✅ Frontend
**文件**: `src-ui/src/App.tsx`
```typescript
// 确保路由配置正确
const router = createBrowserRouter([
    {
        path: '/',
        element: <Layout />,
        children: [
            // ... 其他路由
            {
                path: 'disk-mapping',
                element: <DiskMappingPage />,
                errorElement: <ErrorBoundary />
            },
            // ... 其他路由
        ]
    }
])
```

### T024: 添加错误边界组件 ✅ Frontend
**文件**: `src-ui/src/components/ErrorBoundary.tsx`
```typescript
export class ErrorBoundary extends Component {
    state = { hasError: false, error: null }

    static getDerivedStateFromError(error) {
        return { hasError: true, error }
    }

    componentDidCatch(error, errorInfo) {
        console.error('页面崩溃:', error, errorInfo)
    }

    render() {
        if (this.state.hasError) {
            return (
                <div className="error-page">
                    <h1>页面加载失败</h1>
                    <p>{this.state.error?.message}</p>
                    <button onClick={() => window.location.reload()}>
                        刷新页面
                    </button>
                </div>
            )
        }

        return this.props.children
    }
}
```

### T025: 修复部署页 docker-compose 预览 ✅ Frontend
**文件**: `src-ui/src/pages/DeployPage.tsx`
```typescript
// 加载时同步硬件设备到预览
useEffect(() => {
    const loadDeploymentConfig = async () => {
        // 1. 获取已保存的硬件设备
        const devices = await invoke('get_saved_hardware_devices')

        // 2. 生成包含设备的 docker-compose.yml
        const compose = DockerComposeGenerator.generateCompose({
            // ... 其他配置
            devices: devices.map(d => d.device_path),
        })

        setComposeYaml(compose)
    }

    loadDeploymentConfig()
}, [])
```

---

## Phase 4: 系统流程优化 (T031-T035)

### T031: 删除配置冲突解决页面 ✅ Frontend
**文件**: `src-ui/src/App.tsx`
```typescript
// 删除路由
// 删除: { path: 'conflict-resolution', element: <ConflictResolutionPage /> }
```

### T032: 删除页面文件 ✅ Frontend
```bash
rm src-ui/src/pages/ConflictResolutionPage.tsx
rm src-ui/src/components/ConflictResolver.tsx
```

### T033: 更新导航菜单 ✅ Frontend
**文件**: `src-ui/src/components/Layout.tsx`
```typescript
const menuItems = [
    { path: '/hardware', label: '硬件检测', icon: '🖥️' },
    { path: '/cameras', label: '摄像头发现', icon: '📷' },
    { path: '/config-editor', label: '配置编辑器', icon: '📝' },
    { path: '/disk-mapping', label: '磁盘映射', icon: '💾' },
    { path: '/deploy', label: '部署管理', icon: '🚀' },
    { path: '/rollback', label: '回滚管理', icon: '↩️' },
    // 删除: { path: '/conflict-resolution', label: '冲突解决', icon: '⚠️' },
]
```

### T034: 添加系统状态检查 ✅ Backend
**文件**: `src-tauri/src/commands/system.rs`
```rust
/// 检查系统各组件状态
#[tauri::command]
pub async fn check_system_status() -> Result<SystemStatus, AppError> {
    Ok(SystemStatus {
        hardware_devices: count_saved_devices().await?,
        docker_compose_exists: check_docker_compose_exists().await?,
        config_valid: check_config_validity().await?,
        docker_running: check_docker_running()?,
    })
}
```

### T035: 添加状态指示器 ✅ Frontend
**文件**: `src-ui/src/components/SystemStatus.tsx`
```typescript
export const SystemStatus = () => {
    const [status, setStatus] = useState<SystemStatus | null>(null)

    useEffect(() => {
        const checkStatus = async () => {
            const result = await invoke('check_system_status')
            setStatus(result)
        }

        checkStatus()
        const interval = setInterval(checkStatus, 30000) // 每30秒检查

        return () => clearInterval(interval)
    }, [])

    return (
        <div className="system-status">
            <StatusIndicator
                label="硬件设备"
                value={status?.hardware_devices || 0}
                ok={status?.hardware_devices > 0}
            />
            <StatusIndicator
                label="Docker Compose"
                ok={status?.docker_compose_exists}
            />
            <StatusIndicator
                label="配置文件"
                ok={status?.config_valid}
            />
            <StatusIndicator
                label="Docker"
                ok={status?.docker_running}
            />
        </div>
    )
}
```

---

## 测试清单

### 单元测试
- [ ] `cargo test --lib` 全部通过
- [ ] `npm test` 全部通过

### 集成测试
- [ ] 摄像头扫描并保存结果
- [ ] 页面切换后结果恢复
- [ ] 配置验证检测错误
- [ ] 硬件设备成功部署
- [ ] 卷映射页面正常访问

### 端到端测试
- [ ] 完整部署流程测试
- [ ] 错误配置被正确拦截
- [ ] 系统状态实时更新

## 提交策略

每个 Phase 完成后独立提交：
1. `fix: 修复摄像头发现页网络检测和结果持久化`
2. `fix: 强化配置验证服务，添加完整错误检测`
3. `fix: 修复部署页硬件集成和卷映射崩溃`
4. `refactor: 删除冗余页面，优化系统流程`

---

**执行命令**: `/openspec:apply fix-critical-bugs-and-workflow`
**预计时长**: 6-7小时连续开发