# Feature Specification: 修复硬件设备插入 docker-compose.yml 功能

**ID**: fix-hardware-device-insertion
**Status**: 🔴 Critical Bug
**Priority**: P0
**Created**: 2025-10-16
**Updated**: 2025-10-16

## 📋 问题描述

### 用户报告
用户反馈:"在我测试该项目时,出现一个错误,我设置了硬件后,无法自动把相关参数插入到docker-compose.yml文件里"

### 影响范围
- **严重程度**: 🔴 Critical - 核心功能完全失效
- **影响组件**:
  - Backend: `src-tauri/src/commands/deploy.rs::update_docker_compose_devices()`
  - Frontend: `src-ui/src/pages/HardwarePage.tsx`
  - Config: docker-compose.yml 生成逻辑
- **受影响用户**: 所有用户(100%)

## 🔍 根本原因分析

### 1. 文件路径问题
**问题**: `update_docker_compose_devices()` 函数查找 docker-compose.yml 的逻辑可能有误

```rust
// 当前实现 (deploy.rs:489-503)
let possible_paths = vec![
    default_compose_path.to_string_lossy().to_string(),  // ~/.config/frigate-config-tool/frigate-docker-compose.yml
    format!("{}/frigate/docker-compose.yml", std::env::var("HOME").unwrap_or_default()),
    "/opt/frigate/docker-compose.yml".to_string(),
    "docker-compose.frigate.yml".to_string(),
];
```

**分析**:
- 如果用户的 docker-compose.yml 不在这些路径中,函数会创建新文件
- 但用户可能已经有自己的 docker-compose.yml 在项目根目录或其他位置
- 函数没有提示用户实际使用的文件路径

### 2. YAML 解析问题
**问题**: 如果现有 docker-compose.yml 格式不标准,可能导致解析失败

```rust
// 当前实现 (deploy.rs:528-529)
let mut yaml: serde_yaml::Value = serde_yaml::from_str(&content)
    .map_err(|e| AppError::Deployment(format!("Failed to parse docker-compose.yml: {}", e)))?;
```

**分析**:
- 如果 YAML 语法错误,整个函数会失败
- 错误信息不会返回给用户(只有 warn!)
- 用户看不到任何失败提示

### 3. 服务名称匹配问题
**问题**: 函数硬编码查找 `services.frigate`,但用户的服务名可能不同

```rust
// 当前实现 (deploy.rs:532-533)
if let Some(services) = yaml.get_mut("services") {
    if let Some(service) = services.get_mut("frigate") {
```

**分析**:
- 用户的服务可能叫 `frigate-nvr`, `frigate-main`, 或其他名称
- 如果找不到 `frigate` 服务,函数会返回错误
- 但错误信息被 `warn!` 吃掉了,用户看不到

### 4. 错误处理不当
**问题**: 关键错误被 `warn!` 而非返回给用户

```rust
// 当前实现 (deploy.rs:443-446)
if let Err(e) = update_docker_compose_devices().await {
    warn!("Failed to update docker-compose.yml: {}", e);
    // Don't fail the entire operation if docker-compose update fails
}
```

**分析**:
- 即使 docker-compose.yml 更新失败,用户也会看到"已添加设备到配置"的成功提示
- 用户以为设备已添加,实际上完全没有写入
- 这是严重的用户体验问题

### 5. 缺少调试信息
**问题**: 没有足够的日志帮助排查问题

当前日志:
- ✅ `info!("Using Frigate docker-compose.yml at: {}", compose_path);`
- ✅ `info!("Added {} device mappings to docker-compose.yml", device_count);`
- ❌ 缺少: 保存前的 YAML 内容验证
- ❌ 缺少: 保存后的文件验证
- ❌ 缺少: 设备路径存在性检查的日志

## 💡 解决方案

### Phase 1: 诊断增强 (立即实施)

#### 1.1 添加详细的调试日志
```rust
// 在 update_docker_compose_devices() 开始处
info!("=== Starting docker-compose.yml update ===");
info!("Config directory: {:?}", config_dir);
info!("Default compose path: {:?}", default_compose_path);
info!("Loaded {} hardware devices", devices.len());
for device in &devices {
    info!("  Device: {} ({}) - {}", device.device_name, device.device_type, device.device_path);
}

// 在找到 compose_path 后
info!("Selected docker-compose.yml path: {}", compose_path);
info!("File exists: {}", Path::new(&compose_path).exists());

// 在解析 YAML 后
info!("YAML parsed successfully, services count: {}",
    yaml.get("services").and_then(|s| s.as_mapping()).map(|m| m.len()).unwrap_or(0));

// 在找到 frigate 服务后
info!("Found 'frigate' service in docker-compose.yml");

// 在写入文件前
info!("Generated YAML content ({} bytes):", updated_content.len());
info!("First 500 chars: {}", &updated_content[..updated_content.len().min(500)]);

// 在写入文件后
info!("Successfully wrote docker-compose.yml to: {}", compose_path);
info!("=== docker-compose.yml update completed ===");
```

#### 1.2 返回错误给用户
```rust
// 修改 add_device_internal() 函数
pub async fn add_device_internal(
    device_path: String,
    device_type: String,
    device_name: String,
) -> Result<(), AppError> {
    info!("Adding hardware device to config: {} ({})", device_name, device_path);

    // ... 保存到 hardware_devices.json 的代码 ...

    // 修改错误处理
    if let Err(e) = update_docker_compose_devices().await {
        let error_msg = format!("设备已保存到配置,但无法更新 docker-compose.yml: {}", e);
        warn!("{}", error_msg);
        // 返回错误而不是忽略
        return Err(AppError::Deployment(error_msg));
    }

    Ok(())
}
```

#### 1.3 添加用户友好的错误消息
```rust
// 修改 add_hardware_device_to_config() 返回类型
#[derive(Debug, Serialize, Deserialize)]
pub struct AddHardwareDeviceResponse {
    pub success: bool,
    pub message: String,
    pub device_path: String,
    pub device_type: String,
    pub docker_compose_updated: bool,  // 新增
    pub docker_compose_path: Option<String>,  // 新增
    pub warnings: Vec<String>,  // 新增
}
```

### Phase 2: 路径发现改进 (短期)

#### 2.1 更智能的文件查找
```rust
async fn find_docker_compose_file() -> Result<String, AppError> {
    let possible_paths = vec![
        // 1. 项目根目录(最常见)
        "./docker-compose.yml",
        "./docker-compose.yaml",
        "./compose.yml",
        "./compose.yaml",

        // 2. 用户配置目录
        config_dir.join("frigate-docker-compose.yml").to_string_lossy().to_string(),

        // 3. 用户 HOME 目录的 frigate 文件夹
        format!("{}/frigate/docker-compose.yml", home_dir),
        format!("{}/.frigate/docker-compose.yml", home_dir),

        // 4. 系统标准位置
        "/opt/frigate/docker-compose.yml",
        "/etc/frigate/docker-compose.yml",
    ];

    for path in possible_paths {
        if Path::new(&path).exists() {
            info!("Found existing docker-compose.yml at: {}", path);
            return Ok(path);
        }
    }

    // 如果都不存在,返回默认路径
    let default_path = config_dir.join("frigate-docker-compose.yml");
    warn!("No existing docker-compose.yml found, will create new one at: {:?}", default_path);
    Ok(default_path.to_string_lossy().to_string())
}
```

#### 2.2 让用户指定 docker-compose.yml 路径
```rust
// 添加新的 Tauri 命令
#[tauri::command]
pub async fn set_docker_compose_path(
    path: String,
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<(), AppError> {
    // 验证路径
    if !Path::new(&path).exists() {
        return Err(AppError::Deployment(format!("File not found: {}", path)));
    }

    // 保存到配置文件
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Deployment("Failed to get config directory".to_string()))?
        .join("frigate-config-tool");

    tokio::fs::create_dir_all(&config_dir).await?;

    let config_path = config_dir.join("docker_compose_path.txt");
    tokio::fs::write(&config_path, path).await?;

    Ok(())
}

#[tauri::command]
pub async fn get_docker_compose_path(
    _state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Option<String>, AppError> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| AppError::Deployment("Failed to get config directory".to_string()))?
        .join("frigate-config-tool");

    let config_path = config_dir.join("docker_compose_path.txt");

    if config_path.exists() {
        let path = tokio::fs::read_to_string(&config_path).await?;
        Ok(Some(path))
    } else {
        Ok(None)
    }
}
```

### Phase 3: 服务名称灵活性 (短期)

#### 3.1 智能查找 Frigate 服务
```rust
fn find_frigate_service(yaml: &serde_yaml::Value) -> Option<String> {
    if let Some(services) = yaml.get("services").and_then(|s| s.as_mapping()) {
        // 优先查找精确匹配
        if services.contains_key(&serde_yaml::Value::String("frigate".to_string())) {
            return Some("frigate".to_string());
        }

        // 查找包含 frigate 的服务名
        for (key, _value) in services {
            if let Some(service_name) = key.as_str() {
                if service_name.to_lowercase().contains("frigate") {
                    info!("Found Frigate service with name: {}", service_name);
                    return Some(service_name.to_string());
                }
            }
        }

        // 查找使用 frigate 镜像的服务
        for (key, value) in services {
            if let Some(service_map) = value.as_mapping() {
                if let Some(image) = service_map.get(&serde_yaml::Value::String("image".to_string())) {
                    if let Some(image_str) = image.as_str() {
                        if image_str.contains("frigate") {
                            if let Some(service_name) = key.as_str() {
                                info!("Found Frigate service by image: {}", service_name);
                                return Some(service_name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
```

### Phase 4: 前端改进 (中期)

#### 4.1 添加 docker-compose.yml 路径选择器
```typescript
// HardwarePage.tsx 中添加
const [dockerComposePath, setDockerComposePath] = useState<string | null>(null)

useEffect(() => {
  // 获取当前的 docker-compose.yml 路径
  safeInvoke<string | null>('get_docker_compose_path').then(path => {
    setDockerComposePath(path)
  })
}, [])

// UI 组件
{!isInTauriEnv && (
  <Card className="mb-4">
    <h3>Docker Compose 配置文件</h3>
    <p className="text-sm text-gray-600 mb-2">
      当前路径: {dockerComposePath || '自动检测'}
    </p>
    <Button onClick={() => {/* 打开文件选择器 */}}>
      选择 docker-compose.yml 文件
    </Button>
  </Card>
)}
```

#### 4.2 显示详细的操作结果
```typescript
const handlePresetAdd = async () => {
  // ...
  try {
    const result = await safeInvoke<AddHardwareDeviceResponse>('add_hardware_device_to_config', {
      devicePath: preset.devicePath,
      deviceType: preset.type,
      deviceName: preset.name
    })

    if (result.docker_compose_updated) {
      setAddSuccess(`✅ 已添加 ${preset.name} 到配置和 docker-compose.yml`)
      if (result.docker_compose_path) {
        console.log('Updated docker-compose.yml at:', result.docker_compose_path)
      }
    } else {
      setAddSuccess(`⚠️ 已添加 ${preset.name} 到配置,但 docker-compose.yml 更新失败`)
      if (result.warnings.length > 0) {
        console.warn('Warnings:', result.warnings)
      }
    }
  } catch (err) {
    alert(`添加失败: ${err}`)
  }
}
```

## 🎯 实施计划

### Phase 1: 立即诊断 (1-2小时)
- [x] 添加详细的调试日志到 `update_docker_compose_devices()`
- [x] 修改错误处理逻辑,返回错误给用户
- [x] 更新响应类型,包含更多诊断信息
- [ ] 测试并收集日志

### Phase 2: 路径改进 (2-3小时)
- [ ] 实现智能文件查找逻辑
- [ ] 添加 `set_docker_compose_path` 和 `get_docker_compose_path` 命令
- [ ] 更新前端以支持路径选择
- [ ] 测试不同路径场景

### Phase 3: 服务查找 (1小时)
- [ ] 实现 `find_frigate_service()` 函数
- [ ] 更新 `update_docker_compose_devices()` 使用新逻辑
- [ ] 测试不同服务名称

### Phase 4: 前端优化 (2小时)
- [ ] 添加路径选择 UI
- [ ] 添加详细的成功/失败提示
- [ ] 添加"查看 docker-compose.yml"按钮
- [ ] 测试用户体验

## ✅ 验收标准

### 功能性
- [ ] 用户添加硬件后,设备必须正确插入 docker-compose.yml
- [ ] 如果失败,用户必须看到清晰的错误提示
- [ ] 支持自定义 docker-compose.yml 路径
- [ ] 支持不同的服务名称(不仅仅是 "frigate")

### 可观测性
- [ ] 完整的日志记录每一步操作
- [ ] 用户可以查看实际使用的 docker-compose.yml 路径
- [ ] 用户可以查看生成的 YAML 内容

### 兼容性
- [ ] 支持标准的 docker-compose.yml 格式
- [ ] 支持 Compose v2 和 v3 格式
- [ ] 不破坏现有的用户配置

## 🧪 测试用例

### 1. 基本功能测试
```bash
# 测试用例 1: 默认路径
1. 删除所有可能的 docker-compose.yml
2. 添加一个 Intel GPU
3. 验证文件被创建在 ~/.config/frigate-config-tool/frigate-docker-compose.yml
4. 验证 devices 字段包含 /dev/dri/renderD128

# 测试用例 2: 现有文件
1. 在项目根目录创建 docker-compose.yml
2. 添加一个 Hailo TPU
3. 验证文件被更新(不是创建新文件)
4. 验证 devices 和 privileged 字段正确

# 测试用例 3: NVIDIA GPU
1. 添加 NVIDIA GPU
2. 验证没有 devices 字段
3. 验证有 deploy.resources.reservations 字段
```

### 2. 错误场景测试
```bash
# 测试用例 4: YAML 语法错误
1. 创建一个语法错误的 docker-compose.yml
2. 尝试添加硬件
3. 验证用户看到明确的错误提示

# 测试用例 5: 没有 frigate 服务
1. 创建一个不包含 frigate 服务的 docker-compose.yml
2. 尝试添加硬件
3. 验证用户看到明确的错误提示
```

## 📚 参考资料

- [Frigate 官方安装文档](https://docs.frigate.video/frigate/installation)
- [Docker Compose 文件规范](https://docs.docker.com/compose/compose-file/)
- [Tauri 错误处理最佳实践](https://tauri.app/v1/guides/features/command#error-handling)

## 🔗 相关 Issues

- User Report: "设置了硬件后,无法自动把相关参数插入到docker-compose.yml文件里"
- Related Fix: [PR #xxx] 修复 Frigate 2025 兼容性问题

---

**Next Steps**:
1. ✅ Review this specification
2. ⏳ Implement Phase 1 diagnostics
3. ⏳ Collect user logs
4. ⏳ Implement remaining phases
