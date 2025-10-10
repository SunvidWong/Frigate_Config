# E2E 测试指南

## 概述

本目录包含使用Playwright编写的端到端（E2E）测试。这些测试验证完整的用户工作流程。

## 测试文件

- `hardware-page.spec.ts` - 硬件检测页面测试（User Story 1）
- `cameras-page.spec.ts` - 相机配置页面测试（User Story 1）
- `conflict-resolution.spec.ts` - 冲突解决工作流测试（User Story 2）

## 前置要求

1. 已安装Node.js依赖：
   ```bash
   cd src-ui
   npm install
   ```

2. 已安装Playwright浏览器：
   ```bash
   npx playwright install chromium
   ```

## 运行测试

### 方法1：自动启动开发服务器

Playwright配置会自动启动Tauri开发服务器：

```bash
# 从项目根目录运行
npm run test:e2e
```

### 方法2：手动启动开发服务器

如果遇到服务器启动问题，可以手动启动：

```bash
# 终端1：启动Tauri开发服务器
npm run dev

# 终端2：运行测试
npm run test:e2e
```

### 调试模式

使用Playwright Inspector进行调试：

```bash
npm run test:e2e:debug
```

### UI模式

使用交互式UI运行测试：

```bash
npm run test:e2e:ui
```

## 测试配置

测试配置位于项目根目录的 `playwright.config.ts`：

- **测试超时**: 30秒
- **基础URL**: http://localhost:1420 (Tauri默认端口)
- **浏览器**: Chromium
- **失败时截图**: 启用
- **失败时录制视频**: 启用

## 测试报告

测试运行后，报告生成在：

- HTML报告: `test-results/html/`
- JSON报告: `test-results/results.json`

查看HTML报告：

```bash
npx playwright show-report test-results/html
```

## 编写新测试

### 测试结构

```typescript
import { test, expect } from '@playwright/test';

test.describe('功能名称', () => {
  test.beforeEach(async ({ page }) => {
    // 导航到应用
    await page.goto('http://localhost:1420');
    await page.waitForLoadState('networkidle');
  });

  test('测试场景描述', async ({ page }) => {
    // 测试步骤
    await page.click('text=按钮文本');

    // 断言
    await expect(page.locator('[data-testid="element"]')).toBeVisible();
  });
});
```

### 最佳实践

1. **使用data-testid属性**：为测试目标元素添加`data-testid`属性
2. **等待网络空闲**：使用`waitForLoadState('networkidle')`
3. **明确的断言**：使用具体的expect断言而不是通用检查
4. **独立测试**：每个测试应该独立运行，不依赖其他测试的状态
5. **清理状态**：在beforeEach中重置状态

## 常见问题

### 测试超时

如果测试超时，检查：
- Tauri应用是否正常启动
- 端口1420是否被占用
- 增加`playwright.config.ts`中的timeout值

### 元素找不到

- 确认元素已添加`data-testid`属性
- 使用Playwright Inspector查看DOM结构
- 检查元素是否在正确的时间加载

### 开发服务器启动失败

- 检查src-ui目录的依赖是否安装
- 查看是否有其他进程占用端口
- 手动启动服务器并使用reuseExistingServer选项

## CI/CD集成

在GitHub Actions中运行测试：

```yaml
- name: 运行E2E测试
  run: |
    npm install
    cd src-ui && npm install && cd ..
    npx playwright install --with-deps chromium
    npm run test:e2e
```

## 相关文档

- [Playwright官方文档](https://playwright.dev/)
- [Tauri测试指南](https://tauri.app/v1/guides/testing/)
- 项目测试策略：`/docs/testing-strategy.md`
