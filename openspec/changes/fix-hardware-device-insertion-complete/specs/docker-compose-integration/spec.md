# Spec Delta: Docker Compose Integration - 硬件设备插入完整修复

**Change ID**: fix-hardware-device-insertion-complete
**Capability**: docker-compose-integration
**Type**: Enhancement & Bug Fix

## MODIFIED Requirements

### Requirement: Docker Compose 文件路径发现

系统 SHALL 能够智能地查找和定位用户的 docker-compose.yml 文件,支持多种路径和文件名变体。

#### Scenario: 自动路径搜索

- **GIVEN** 用户没有手动指定 docker-compose.yml 路径
- **WHEN** 系统需要更新 docker-compose.yml
- **THEN** 系统应按以下顺序搜索:
  1. `./docker-compose.yml` (项目根目录)
  2. `./docker-compose.yaml` (YAML 扩展名变体)
  3. `~/.config/frigate-config-tool/frigate-docker-compose.yml` (用户配置目录)
  4. `~/frigate/docker-compose.yml` (用户 HOME 目录下的 frigate 文件夹)
  5. `/opt/frigate/docker-compose.yml` (系统标准位置)
  6. `./docker-compose.frigate.yml` (项目根目录下的命名变体)
- **AND** 系统应在日志中记录每个搜索路径及其存在性
- **AND** 系统应使用找到的第一个存在的文件

#### Scenario: 未找到文件时创建新文件

- **GIVEN** 所有搜索路径都不存在 docker-compose.yml
- **WHEN** 系统需要更新 docker-compose.yml
- **THEN** 系统应在 `~/.config/frigate-config-tool/frigate-docker-compose.yml` 创建新文件
- **AND** 新文件应包含 Frigate 官方推荐的模板内容
- **AND** 系统应在日志中记录新文件创建

### Requirement: 用户自定义 Docker Compose 路径

系统 SHALL 允许用户手动指定 docker-compose.yml 文件的路径,并持久化此配置。

#### Scenario: 设置自定义路径

- **GIVEN** 用户选择了一个 docker-compose.yml 文件路径
- **WHEN** 调用 `set_docker_compose_path(path)` 命令
- **THEN** 系统应验证文件存在
- **AND** 如果文件存在,系统应保存路径到 `~/.config/frigate-config-tool/docker_compose_path.txt`
- **AND** 系统应返回成功响应
- **AND** 如果文件不存在,系统应返回错误 "File not found: {path}"

#### Scenario: 读取自定义路径

- **GIVEN** 用户之前设置了自定义 docker-compose.yml 路径
- **WHEN** 调用 `get_docker_compose_path()` 命令
- **THEN** 系统应从 `~/.config/frigate-config-tool/docker_compose_path.txt` 读取并返回路径
- **AND** 如果未设置自定义路径,系统应返回 `None`

#### Scenario: 使用自定义路径优先

- **GIVEN** 用户设置了自定义 docker-compose.yml 路径
- **WHEN** 系统需要更新 docker-compose.yml
- **THEN** 系统应优先使用自定义路径
- **AND** 系统应在日志中标明使用的是用户自定义路径
- **AND** 系统不应搜索其他路径

#### Scenario: 重置为自动检测

- **GIVEN** 用户之前设置了自定义路径
- **WHEN** 用户删除 `~/.config/frigate-config-tool/docker_compose_path.txt` 文件
- **THEN** 系统应恢复自动搜索模式
- **AND** 下次更新时应使用自动路径发现逻辑

## ADDED Requirements

### Requirement: 智能 Frigate 服务识别

系统 SHALL 能够识别 docker-compose.yml 中的 Frigate 服务,即使服务名称不是标准的 "frigate"。

#### Scenario: 精确匹配服务名

- **GIVEN** docker-compose.yml 中有名为 "frigate" 的服务
- **WHEN** 系统查找 Frigate 服务
- **THEN** 系统应立即返回 "frigate" 作为服务名
- **AND** 不需要进行进一步的搜索

#### Scenario: 模糊匹配服务名

- **GIVEN** docker-compose.yml 中没有名为 "frigate" 的服务
- **AND** 存在服务名包含 "frigate" 的服务(如 "frigate-nvr", "frigate-main")
- **WHEN** 系统查找 Frigate 服务
- **THEN** 系统应返回第一个包含 "frigate" 的服务名(不区分大小写)
- **AND** 系统应在日志中记录找到的服务名

#### Scenario: 通过镜像识别 Frigate 服务

- **GIVEN** docker-compose.yml 中的服务名不包含 "frigate"
- **AND** 存在使用 Frigate Docker 镜像的服务(镜像名包含 "frigate")
- **WHEN** 系统查找 Frigate 服务
- **THEN** 系统应通过检查 `image` 字段识别 Frigate 服务
- **AND** 系统应返回该服务的名称
- **AND** 系统应在日志中记录通过镜像识别的过程

#### Scenario: 未找到 Frigate 服务

- **GIVEN** docker-compose.yml 中不存在任何 Frigate 相关的服务
- **WHEN** 系统查找 Frigate 服务
- **THEN** 系统应返回 `None`
- **AND** 系统应在日志中列出所有可用的服务名称
- **AND** 系统应返回清晰的错误消息: "No 'frigate' service found in docker-compose.yml. Available services: {service_list}"

### Requirement: 详细的硬件插入响应

系统 SHALL 在硬件设备添加操作后返回详细的诊断信息,包括 docker-compose.yml 更新状态。

#### Scenario: 成功添加硬件并更新 Docker Compose

- **GIVEN** 硬件设备有效且 docker-compose.yml 可更新
- **WHEN** 用户添加硬件设备
- **THEN** 响应应包含:
  - `success: true`
  - `message: "已添加设备 {name} 到配置"`
  - `device_path: "{path}"`
  - `device_type: "{type}"`
  - `docker_compose_updated: true`
  - `docker_compose_path: "{actual_path}"`
  - `frigate_service_name: "{service_name}"`
  - `warnings: []`

#### Scenario: 添加硬件但 Docker Compose 更新失败

- **GIVEN** 硬件设备有效但 docker-compose.yml 更新失败(如服务未找到)
- **WHEN** 用户添加硬件设备
- **THEN** 系统应返回错误
- **AND** 错误消息应包含:
  - 设备已保存到配置文件的确认
  - docker-compose.yml 更新失败的具体原因
  - 查看日志的建议

#### Scenario: 包含警告的成功添加

- **GIVEN** 硬件设备添加成功但有非致命问题
- **WHEN** 用户添加硬件设备
- **THEN** 响应应包含:
  - `success: true`
  - `docker_compose_updated: true`
  - `warnings: ["{warning_message}", ...]`
- **AND** 警告可能包括:
  - 设备文件路径不存在(但仍添加到配置)
  - 使用了备用路径或服务名
  - 其他非致命问题

### Requirement: Docker Compose 文件查看功能

系统 SHALL 提供查看当前使用的 docker-compose.yml 文件内容的能力。

#### Scenario: 查看 Docker Compose 文件

- **GIVEN** 系统已确定要使用的 docker-compose.yml 路径
- **WHEN** 用户请求查看 docker-compose.yml
- **THEN** 系统应返回:
  - 文件的完整路径
  - 文件的完整内容
  - 文件是否存在的状态

#### Scenario: Docker Compose 文件不存在

- **GIVEN** docker-compose.yml 文件在预期路径不存在
- **WHEN** 用户请求查看 docker-compose.yml
- **THEN** 系统应返回:
  - 预期的文件路径
  - `exists: false` 状态
  - 建议创建文件的提示

## MODIFIED Requirements (Updated from Phase 1)

### Requirement: Docker Compose 设备映射更新

系统 SHALL 能够将配置的硬件设备正确写入到 docker-compose.yml 的 Frigate 服务配置中。

**Changes from Phase 1:**
- 不再硬编码查找 "frigate" 服务,使用 `find_frigate_service()` 动态查找
- 优先使用用户自定义的 docker-compose.yml 路径
- 返回更详细的诊断信息

#### Scenario: 更新非标准服务名的 Docker Compose

- **GIVEN** docker-compose.yml 中的 Frigate 服务名为 "frigate-nvr"
- **AND** 用户添加了一个硬件设备
- **WHEN** 系统更新 docker-compose.yml
- **THEN** 系统应能识别 "frigate-nvr" 服务
- **AND** 系统应将设备映射添加到 "frigate-nvr" 服务的 `devices` 字段
- **AND** 系统应在日志中记录使用的服务名

#### Scenario: 使用自定义路径更新 Docker Compose

- **GIVEN** 用户设置了自定义 docker-compose.yml 路径为 `/home/user/projects/frigate/compose.yml`
- **AND** 用户添加了一个硬件设备
- **WHEN** 系统更新 docker-compose.yml
- **THEN** 系统应读取和写入 `/home/user/projects/frigate/compose.yml`
- **AND** 系统不应搜索其他标准路径
- **AND** 系统应在日志中标明使用自定义路径

## Frontend Integration

### Requirement: Docker Compose 路径配置 UI

前端 SHALL 提供用户界面让用户配置 docker-compose.yml 文件路径。

#### Scenario: 显示当前路径

- **GIVEN** 用户打开硬件配置页面
- **WHEN** 页面加载
- **THEN** UI 应显示:
  - 当前使用的 docker-compose.yml 路径
  - 路径来源标识(自动检测 vs 用户自定义)
  - 文件是否存在的状态指示器

#### Scenario: 选择自定义路径

- **GIVEN** 用户在硬件配置页面
- **WHEN** 用户点击"选择文件"按钮
- **THEN** 应打开文件选择对话框
- **AND** 用户选择文件后,系统应调用 `set_docker_compose_path()`
- **AND** UI 应更新显示新的路径
- **AND** 应显示成功提示

#### Scenario: 重置为自动检测

- **GIVEN** 用户已设置了自定义路径
- **WHEN** 用户点击"重置为自动检测"按钮
- **THEN** 系统应删除保存的自定义路径
- **AND** UI 应更新显示为"自动检测"模式
- **AND** 应显示路径搜索的优先级列表

### Requirement: 硬件添加结果的详细反馈

前端 SHALL 向用户显示硬件设备添加操作的详细结果和状态。

#### Scenario: 显示成功信息

- **GIVEN** 硬件设备成功添加且 docker-compose.yml 已更新
- **WHEN** 操作完成
- **THEN** UI 应显示:
  - ✅ 成功消息: "已添加 {device_name} 到配置"
  - 设备路径和类型
  - Docker Compose 更新确认: "已更新 {compose_path}"
  - 使用的 Frigate 服务名称
  - (可选) "查看 docker-compose.yml" 链接

#### Scenario: 显示部分失败信息

- **GIVEN** 硬件设备保存成功但 docker-compose.yml 更新失败
- **WHEN** 操作完成
- **THEN** UI 应显示:
  - ⚠️ 警告消息: "设备已保存到配置,但无法更新 docker-compose.yml"
  - 失败的具体原因
  - "查看日志"和"手动修复"引导链接

#### Scenario: 显示警告信息

- **GIVEN** 操作成功但有警告
- **WHEN** 操作完成
- **THEN** UI 应显示:
  - ✅ 成功消息
  - ⚠️ 警告列表
  - 每个警告的详细说明

### Requirement: 查看 Docker Compose 文件功能

前端 SHALL 提供查看当前 docker-compose.yml 文件内容的功能。

#### Scenario: 查看文件内容

- **GIVEN** 用户在硬件配置页面
- **WHEN** 用户点击"查看 docker-compose.yml"按钮
- **THEN** 应打开模态框显示:
  - 文件的完整路径
  - 文件的内容(语法高亮)
  - Frigate 服务和 devices 部分高亮显示

#### Scenario: 文件不存在时的提示

- **GIVEN** docker-compose.yml 文件不存在
- **WHEN** 用户点击"查看 docker-compose.yml"按钮
- **THEN** 应显示:
  - 提示文件不存在
  - 预期的文件路径
  - 建议用户创建文件或添加硬件(会自动创建)

## Testing Requirements

### Requirement: 路径发现和服务识别测试

系统 SHALL 包含全面的测试覆盖路径发现和服务识别功能。

#### Scenario: 单元测试覆盖

- **GIVEN** 实现了路径发现和服务识别功能
- **WHEN** 运行单元测试
- **THEN** 应包含测试:
  - 每个路径搜索位置的优先级
  - 自定义路径的保存和读取
  - 精确、模糊和镜像匹配的服务识别
  - 未找到服务的错误处理

#### Scenario: 集成测试覆盖

- **GIVEN** 实现了完整的硬件插入流程
- **WHEN** 运行集成测试
- **THEN** 应包含测试:
  - 使用不同路径的 docker-compose.yml
  - 使用不同服务名称的 docker-compose.yml
  - 自定义路径配置和使用
  - 完整的成功和失败场景

#### Scenario: E2E 测试覆盖

- **GIVEN** 实现了前端和后端集成
- **WHEN** 运行 E2E 测试
- **THEN** 应包含测试:
  - 完整的路径配置 UI 交互
  - 硬件添加和结果显示
  - 查看 docker-compose.yml 功能
  - 错误场景和手动修复引导

---

**变更类型总结**:
- **MODIFIED Requirements**: 2 (路径发现、设备映射更新)
- **ADDED Requirements**: 6 (智能服务识别、详细响应、文件查看、前端 UI、结果反馈、测试)

**影响范围**:
- Backend: `src-tauri/src/commands/deploy.rs`
- Frontend: `src-ui/src/pages/HardwarePage.tsx`
- Tests: 新增多个测试文件

**向后兼容性**: ✅ 完全兼容,只增强功能不破坏现有行为
