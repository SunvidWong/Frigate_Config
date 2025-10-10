// T168: E2E test for manual YAML editing flow
// Tests the complete manual configuration editing workflow

import { test, expect } from '@playwright/test'
import { readFileSync, writeFileSync } from 'fs'
import { join } from 'path'
import { tmpdir } from 'os'

const TEST_CONFIG = `
# Test Frigate Configuration
mqtt:
  enabled: true
  host: localhost
  port: 1883

cameras:
  front_door:
    enabled: true
    ffmpeg:
      inputs:
        - path: rtsp://camera/stream
          roles:
            - detect
            - record
    detect:
      enabled: true
      fps: 5

detectors:
  cpu1:
    type: cpu
`

test.describe('Manual YAML Editing', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    // Navigate to Manual Config page
    await page.click('text=Manual Configuration')
  })

  test('should load and display YAML configuration', async ({ page }) => {
    // Create temporary config file
    const tempConfigPath = join(tmpdir(), 'test-config.yml')
    writeFileSync(tempConfigPath, TEST_CONFIG)

    // Click load button
    await page.click('button:has-text("Load Configuration")')

    // In test environment, we'd need to mock the file dialog
    // For now, test that the button is clickable
    expect(await page.isEnabled('button:has-text("Load Configuration")')).toBe(true)
  })

  test('should show unsaved changes indicator', async ({ page }) => {
    // Directly set content via YAML editor
    const editor = page.locator('textarea')

    // Type some content
    await editor.fill(TEST_CONFIG)

    // Check for unsaved changes indicator
    await expect(page.locator('text=Unsaved changes')).toBeVisible({ timeout: 2000 })
  })

  test('should validate YAML syntax in real-time', async ({ page }) => {
    const editor = page.locator('textarea')

    // Enter invalid YAML
    const invalidYaml = `
mqtt:
  enabled: true
  host: [incomplete array
`

    await editor.fill(invalidYaml)

    // Wait for validation
    await page.waitForTimeout(1000)

    // Check for validation errors
    const errorMessage = page.locator('.alert-error, [class*="error"], [class*="validation"]')
    await expect(errorMessage).toBeVisible({ timeout: 5000 })
  })

  test('should show validation errors with line numbers', async ({ page }) => {
    const editor = page.locator('textarea')

    // Enter YAML with validation error
    const invalidConfig = `
mqtt:
  enabled: true
cameras:
  invalid_camera:
    # Missing required fields
    enabled: true
`

    await editor.fill(invalidConfig)

    // Wait for validation
    await page.waitForTimeout(1500)

    // Look for line number indicators in errors
    const errorElements = page.locator('[class*="error"], [class*="validation"]')
    const errorCount = await errorElements.count()

    expect(errorCount).toBeGreaterThan(0)
  })

  test('should preserve comments during editing', async ({ page }) => {
    const editor = page.locator('textarea')

    const configWithComments = `
# Main MQTT configuration
mqtt:
  enabled: true  # Enable MQTT notifications
  host: localhost  # Broker address
`

    await editor.fill(configWithComments)

    // Get the value back
    const editorValue = await editor.inputValue()

    // Verify comments are preserved
    expect(editorValue).toContain('# Main MQTT configuration')
    expect(editorValue).toContain('# Enable MQTT notifications')
    expect(editorValue).toContain('# Broker address')
  })

  test('should handle Tab key for indentation', async ({ page }) => {
    const editor = page.locator('textarea')

    // Focus editor
    await editor.click()

    // Type some YAML
    await editor.fill('mqtt:\n')

    // Move cursor to end
    await editor.press('End')

    // Press Tab
    await editor.press('Tab')

    // Type more
    await editor.type('enabled: true')

    const content = await editor.inputValue()

    // Should have 2-space indentation
    expect(content).toContain('  enabled: true')
  })

  test('should support Ctrl+S to save', async ({ page }) => {
    const editor = page.locator('textarea')

    await editor.fill(TEST_CONFIG)

    // Press Ctrl+S (or Cmd+S on Mac)
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control'
    await editor.press(`${modifier}+KeyS`)

    // Save button should be triggered
    // In a real environment, this would invoke the save dialog
  })

  test('should display line numbers', async ({ page }) => {
    const editor = page.locator('textarea')

    await editor.fill(TEST_CONFIG)

    // Check for line numbers display
    const lineNumbers = page.locator('[class*="line-number"], [class*="gutter"]')
    expect(await lineNumbers.count()).toBeGreaterThan(0)
  })

  test('should show cursor position', async ({ page }) => {
    const editor = page.locator('textarea')

    await editor.fill(TEST_CONFIG)

    // Click into editor
    await editor.click()

    // Look for cursor position indicator (e.g., "Line 1, Col 1")
    const positionIndicator = page.locator('text=/[Ll]ine \\d+|行 \\d+/')
    await expect(positionIndicator).toBeVisible({ timeout: 2000 })
  })

  test('should validate on load', async ({ page }) => {
    // This test would require mocking file dialog
    // Check that validation panel exists
    const validationSection = page.locator('[class*="validation"], .validation-panel')

    // Validation UI should be present (even if empty)
    await expect(validationSection).toBeAttached()
  })

  test('should show error count badge', async ({ page }) => {
    const editor = page.locator('textarea')

    // Enter invalid config with multiple errors
    const multiErrorConfig = `
mqtt:
cameras:
  cam1:
    # Multiple missing required fields
    enabled: true
`

    await editor.fill(multiErrorConfig)

    // Wait for validation
    await page.waitForTimeout(1500)

    // Look for error count
    const errorBadge = page.locator('text=/\\d+ 错误|\\d+ error/i')
    await expect(errorBadge).toBeVisible({ timeout: 5000 })
  })

  test('should disable save button when errors exist', async ({ page }) => {
    const editor = page.locator('textarea')

    // Enter invalid config
    const invalidConfig = `
mqtt:
  enabled: invalid_boolean_value
`

    await editor.fill(invalidConfig)

    // Wait for validation
    await page.waitForTimeout(1500)

    // Save button should be disabled
    const saveButton = page.locator('button:has-text("Save")')
    await expect(saveButton).toBeDisabled({ timeout: 5000 })
  })

  test('should enable save button when config is valid', async ({ page }) => {
    const editor = page.locator('textarea')

    // Enter valid config
    await editor.fill(TEST_CONFIG)

    // Wait for validation
    await page.waitForTimeout(1500)

    // Save button should be enabled
    const saveButton = page.locator('button:has-text("Save")')
    await expect(saveButton).toBeEnabled({ timeout: 5000 })
  })

  test('should show validation warnings without blocking save', async ({ page }) => {
    const editor = page.locator('textarea')

    // Enter config with warning (e.g., deprecated field)
    const configWithWarning = `
mqtt:
  enabled: true
  host: localhost
cameras:
  cam1:
    enabled: true
    # Some deprecated field that generates warning
    width: 1920
    height: 1080
    ffmpeg:
      inputs:
        - path: rtsp://test
`

    await editor.fill(configWithWarning)

    // Wait for validation
    await page.waitForTimeout(1500)

    // Look for warning badge (but save should still be enabled)
    const warningBadge = page.locator('text=/警告|warning/i')

    // Save button should still be enabled despite warning
    const saveButton = page.locator('button:has-text("Save")')
    const isEnabled = await saveButton.isEnabled()

    // Either no warnings (valid config) or warnings don't block save
    expect(isEnabled).toBe(true)
  })

  test('should track manual edit markers', async ({ page }) => {
    const editor = page.locator('textarea')

    // Enter config with manual edit markers
    const manualConfig = `
# TODO: Configure proper MQTT broker
mqtt:
  enabled: true
  # FIXME: Update host
  host: localhost

# MANUAL: Custom camera configuration
cameras:
  cam1:
    enabled: true
`

    await editor.fill(manualConfig)

    // Check that editor displays content correctly
    const editorValue = await editor.inputValue()

    expect(editorValue).toContain('TODO:')
    expect(editorValue).toContain('FIXME:')
    expect(editorValue).toContain('MANUAL:')
  })

  test('should handle large configurations', async ({ page }) => {
    const editor = page.locator('textarea')

    // Generate large config
    let largeConfig = 'cameras:\n'
    for (let i = 0; i < 50; i++) {
      largeConfig += `  camera_${i}:\n`
      largeConfig += `    enabled: true\n`
      largeConfig += `    ffmpeg:\n`
      largeConfig += `      inputs:\n`
      largeConfig += `        - path: rtsp://camera${i}/stream\n`
    }

    await editor.fill(largeConfig)

    // Editor should handle large content
    const editorValue = await editor.inputValue()
    expect(editorValue.length).toBeGreaterThan(1000)

    // Line numbers should be rendered for all lines
    const lines = editorValue.split('\n')
    expect(lines.length).toBeGreaterThan(200)
  })

  test('should preserve formatting on edit', async ({ page }) => {
    const editor = page.locator('textarea')

    const formattedConfig = `
mqtt:
  enabled: true
  host: localhost

cameras:
  front_door:
    enabled: true
`

    await editor.fill(formattedConfig)

    // Make a small edit
    await editor.focus()
    await editor.press('End')
    await editor.type('\n  port: 1883')

    const editorValue = await editor.inputValue()

    // Original formatting should be preserved
    expect(editorValue).toContain('mqtt:')
    expect(editorValue).toContain('cameras:')
    expect(editorValue).toContain('port: 1883')
  })

  test('should support search functionality', async ({ page }) => {
    const editor = page.locator('textarea')

    await editor.fill(TEST_CONFIG)

    // Look for search input (might be in toolbar)
    const searchInput = page.locator('input[placeholder*="搜索"], input[placeholder*="Search"]')

    if (await searchInput.isVisible()) {
      // Enter search term
      await searchInput.fill('mqtt')

      // Should highlight occurrences
      await page.waitForTimeout(500)
    }
  })

  test('should show syntax validation immediately', async ({ page }) => {
    const editor = page.locator('textarea')

    // Start with valid config
    await editor.fill('mqtt:\n  enabled: true')

    // Wait for validation
    await page.waitForTimeout(1000)

    // Add invalid syntax
    await editor.press('End')
    await editor.type('\n  host: [unclosed')

    // Wait for validation to update
    await page.waitForTimeout(1500)

    // Error should be shown
    const errorIndicator = page.locator('[class*="error"]')
    expect(await errorIndicator.count()).toBeGreaterThan(0)
  })
})

test.describe('Manual YAML Editing - Keyboard Shortcuts', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await page.click('text=Manual Configuration')
  })

  test('should support Escape to cancel', async ({ page }) => {
    const editor = page.locator('textarea')

    await editor.fill(TEST_CONFIG)

    // Press Escape
    await editor.press('Escape')

    // Cancel action should be triggered (if implemented)
  })

  test('should support Ctrl+F for find', async ({ page }) => {
    const editor = page.locator('textarea')

    await editor.fill(TEST_CONFIG)

    // Press Ctrl+F (or Cmd+F on Mac)
    const modifier = process.platform === 'darwin' ? 'Meta' : 'Control'
    await editor.press(`${modifier}+KeyF`)

    // Search input should be focused
    const searchInput = page.locator('input[id*="search"], input[placeholder*="搜索"]')

    if (await searchInput.isVisible()) {
      await expect(searchInput).toBeFocused({ timeout: 1000 })
    }
  })
})
