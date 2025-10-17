import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright配置文件
 * 用于E2E测试Tauri应用
 */
export default defineConfig({
  // 测试目录
  testDir: './tests/e2e',

  // 完全并行运行测试
  fullyParallel: true,

  // CI上失败时重试
  retries: process.env.CI ? 2 : 0,

  // 并发worker数量
  workers: process.env.CI ? 1 : undefined,

  // 测试报告配置
  reporter: [
    ['html', { outputFolder: 'test-results/html' }],
    ['list'],
    ['json', { outputFile: 'test-results/results.json' }]
  ],

  // 全局超时设置
  timeout: 30000,

  // 期望超时
  expect: {
    timeout: 5000
  },

  // 共享配置
  use: {
    // 基础URL - Tauri开发服务器
    baseURL: 'http://localhost:15000',

    // 收集失败测试的追踪信息
    trace: 'on-first-retry',

    // 截图设置
    screenshot: 'only-on-failure',

    // 视频设置
    video: 'retain-on-failure',
  },

  // 配置浏览器项目
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },

    // 可选：在其他浏览器上测试
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },

    // {
    //   name: 'webkit',
    //   use: { ...devices['Desktop Safari'] },
    // },
  ],

  // Tauri开发服务器配置
  // 注意：Tauri应用需要手动启动
  // 运行测试前执行: npm run dev
  webServer: {
    command: 'cd src-ui && npm run dev',
    url: 'http://localhost:15000',
    reuseExistingServer: !process.env.CI,
    timeout: 120000, // 2分钟启动超时
  },
});
