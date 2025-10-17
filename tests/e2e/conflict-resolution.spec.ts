// T059: E2E test for conflict resolution workflow
// Test the complete conflict resolution user flow

import { test, expect } from '@playwright/test';
import path from 'path';
import fs from 'fs/promises';

test.describe('Conflict Resolution Workflow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:15000');
    await page.waitForLoadState('networkidle');
  });

  test('should detect and display conflicts when merging templates', async ({ page }) => {
    // Navigate to Manual Config page
    await page.click('text=Manual Config');

    // Load an existing configuration
    const existingConfig = `
mqtt:
  enabled: true
  host: mqtt.local
  port: 1883

cameras:
  front_door:
    enabled: true
    fps: 30
`;

    await page.fill('[data-testid="yaml-editor"]', existingConfig);
    await page.click('button:has-text("Save")');

    // Apply a template with conflicts
    await page.click('button:has-text("Apply Template")');
    await page.click('text=Coral TPU Basic');

    // Wait for conflict dialog
    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();

    // Verify conflicts are displayed
    await expect(page.locator('[data-testid="conflict-item"]')).toHaveCount(1);
    await expect(page.locator('text=mqtt.host')).toBeVisible();
    await expect(page.locator('text=mqtt.local')).toBeVisible(); // existing value
    await expect(page.locator('text=mqtt.template')).toBeVisible(); // template value
  });

  test('should allow user to keep existing values', async ({ page }) => {
    // Setup: Load config and trigger conflict
    await page.click('text=Manual Config');

    const config = 'mqtt:\n  host: mqtt.local\n';
    await page.fill('[data-testid="yaml-editor"]', config);
    await page.click('button:has-text("Save")');

    await page.click('button:has-text("Apply Template")');
    await page.selectOption('[data-testid="template-selector"]', 'remote-mqtt-template');

    // Conflict dialog appears
    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();

    // Choose to keep existing value
    await page.click('[data-testid="conflict-resolution-keep"]');
    await page.click('button:has-text("Apply Resolution")');

    // Verify merged config keeps original value
    const editorContent = await page.inputValue('[data-testid="yaml-editor"]');
    expect(editorContent).toContain('host: mqtt.local');
    expect(editorContent).not.toContain('host: mqtt.remote');
  });

  test('should allow user to use template values', async ({ page }) => {
    // Setup
    await page.click('text=Manual Config');

    const config = 'mqtt:\n  host: mqtt.local\n';
    await page.fill('[data-testid="yaml-editor"]', config);
    await page.click('button:has-text("Save")');

    await page.click('button:has-text("Apply Template")');
    await page.selectOption('[data-testid="template-selector"]', 'remote-mqtt-template');

    // Conflict dialog appears
    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();

    // Choose to use template value
    await page.click('[data-testid="conflict-resolution-template"]');
    await page.click('button:has-text("Apply Resolution")');

    // Verify merged config uses template value
    const editorContent = await page.inputValue('[data-testid="yaml-editor"]');
    expect(editorContent).toContain('host: mqtt.remote');
  });

  test('should allow user to provide custom value', async ({ page }) => {
    // Setup
    await page.click('text=Manual Config');

    const config = 'cameras:\n  front_door:\n    fps: 15\n';
    await page.fill('[data-testid="yaml-editor"]', config);
    await page.click('button:has-text("Save")');

    await page.click('button:has-text("Apply Template")');
    await page.selectOption('[data-testid="template-selector"]', 'high-fps-template');

    // Conflict dialog appears
    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();

    // Choose custom value
    await page.click('[data-testid="conflict-resolution-custom"]');
    await page.fill('[data-testid="custom-value-input"]', '25');
    await page.click('button:has-text("Apply Resolution")');

    // Verify merged config uses custom value
    const editorContent = await page.inputValue('[data-testid="yaml-editor"]');
    expect(editorContent).toContain('fps: 25');
  });

  test('should handle multiple conflicts', async ({ page }) => {
    await page.click('text=Manual Config');

    const config = `
mqtt:
  host: mqtt.local
  port: 1883

cameras:
  front_door:
    fps: 15
    resolution: 1920x1080
`;

    await page.fill('[data-testid="yaml-editor"]', config);
    await page.click('button:has-text("Save")');

    await page.click('button:has-text("Apply Template")');
    await page.selectOption('[data-testid="template-selector"]', 'multi-conflict-template');

    // Conflict dialog with multiple conflicts
    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();
    const conflictCount = await page.locator('[data-testid="conflict-item"]').count();
    expect(conflictCount).toBeGreaterThan(1);

    // Resolve each conflict differently
    await page.locator('[data-testid="conflict-item"]').nth(0).locator('[data-testid="conflict-resolution-keep"]').click();
    await page.locator('[data-testid="conflict-item"]').nth(1).locator('[data-testid="conflict-resolution-template"]').click();

    await page.click('button:has-text("Apply Resolution")');

    // Verify mixed resolution
    const editorContent = await page.inputValue('[data-testid="yaml-editor"]');
    expect(editorContent).toContain('host: mqtt.local'); // kept
    // Template value for second conflict should be present
  });

  test('should show conflict severity indicators', async ({ page }) => {
    await page.click('text=Manual Config');

    const config = 'cameras:\n  front_door:\n    fps: 30\n';
    await page.fill('[data-testid="yaml-editor"]', config);
    await page.click('button:has-text("Save")');

    await page.click('button:has-text("Apply Template")');
    await page.selectOption('[data-testid="template-selector"]', 'performance-template');

    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();

    // Verify severity is displayed
    const severityBadge = page.locator('[data-testid="conflict-severity"]');
    await expect(severityBadge).toBeVisible();

    const severityText = await severityBadge.textContent();
    expect(['warning', 'error', 'info']).toContain(severityText?.toLowerCase());
  });

  test('should allow canceling conflict resolution', async ({ page }) => {
    await page.click('text=Manual Config');

    const original_config = 'mqtt:\n  host: mqtt.local\n';
    await page.fill('[data-testid="yaml-editor"]', original_config);
    await page.click('button:has-text("Save")');

    await page.click('button:has-text("Apply Template")');
    await page.selectOption('[data-testid="template-selector"]', 'remote-mqtt-template');

    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();

    // Cancel instead of resolving
    await page.click('button:has-text("Cancel")');

    // Verify original config unchanged
    const editorContent = await page.inputValue('[data-testid="yaml-editor"]');
    expect(editorContent).toBe(original_config);
  });

  test('should preserve comments after conflict resolution', async ({ page }) => {
    await page.click('text=Manual Config');

    const configWithComments = `
# MQTT Settings
mqtt:
  enabled: true  # Enable MQTT
  host: mqtt.local
`;

    await page.fill('[data-testid="yaml-editor"]', configWithComments);
    await page.click('button:has-text("Save")');

    await page.click('button:has-text("Apply Template")');
    await page.selectOption('[data-testid="template-selector"]', 'remote-mqtt-template');

    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();
    await page.click('[data-testid="conflict-resolution-keep"]');
    await page.click('button:has-text("Apply Resolution")');

    // Verify comments preserved
    const editorContent = await page.inputValue('[data-testid="yaml-editor"]');
    expect(editorContent).toContain('# MQTT Settings');
    expect(editorContent).toContain('# Enable MQTT');
  });

  test('should validate resolved configuration', async ({ page }) => {
    await page.click('text=Manual Config');

    const config = 'mqtt:\n  port: 1883\n';
    await page.fill('[data-testid="yaml-editor"]', config);
    await page.click('button:has-text("Save")');

    await page.click('button:has-text("Apply Template")');
    await page.selectOption('[data-testid="template-selector"]', 'invalid-template');

    await expect(page.locator('[data-testid="conflict-dialog"]')).toBeVisible();

    // Provide invalid custom value
    await page.click('[data-testid="conflict-resolution-custom"]');
    await page.fill('[data-testid="custom-value-input"]', 'invalid_port');
    await page.click('button:has-text("Apply Resolution")');

    // Should show validation error
    await expect(page.locator('[data-testid="validation-error"]')).toBeVisible();
    await expect(page.locator('text=Invalid value')).toBeVisible();
  });

  test('should auto-resolve conflicts when possible', async ({ page }) => {
    await page.click('text=Manual Config');

    const config = 'mqtt:\n  enabled: true\n';
    await page.fill('[data-testid="yaml-editor"]', config);
    await page.click('button:has-text("Save")');

    await page.click('button:has-text("Apply Template")');
    await page.selectOption('[data-testid="template-selector"]', 'auto-resolvable-template');

    // If all conflicts are auto-resolvable, dialog should show suggestion
    if (await page.locator('[data-testid="conflict-dialog"]').isVisible()) {
      const autoResolveButton = page.locator('button:has-text("Auto-Resolve All")');
      if (await autoResolveButton.isVisible()) {
        await autoResolveButton.click();

        // Conflicts should be resolved automatically
        await expect(page.locator('[data-testid="conflict-dialog"]')).not.toBeVisible();
      }
    }
  });
});
