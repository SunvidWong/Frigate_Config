/**
 * E2E tests for Disk Mapping page
 * T174 [P] [US6] E2E test for Disk Mapping page
 *
 * Tests the full user journey for disk and volume mapping:
 * - Page navigation and rendering
 * - Disk information display
 * - Volume selection and validation
 * - Path input and validation
 * - Configuration preview
 * - Save and apply configuration
 * - Cross-platform path handling
 * - Error handling and user feedback
 */

import { test, expect, Page } from '@playwright/test';

test.describe('Disk Mapping Page', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the Disk Mapping page
    await page.goto('/disk-mapping');
    await page.waitForLoadState('networkidle');
  });

  test('should display disk mapping page with title', async ({ page }) => {
    // RED PHASE: This test should fail because DiskMappingPage doesn't exist yet

    // Should have page title
    const title = page.locator('h1, h2').filter({ hasText: /磁盘映射|Disk Mapping/i });
    await expect(title).toBeVisible();

    // Should have description or help text
    const description = page.locator('text=/存储卷配置|Configure storage volumes/i');
    await expect(description).toBeVisible();
  });

  test('should list available disks with information', async ({ page }) => {
    // RED PHASE: Test disk information display

    // Wait for disk detection to complete
    await page.waitForSelector('[data-testid="disk-list"]', { timeout: 5000 });

    // Should display at least one disk
    const diskCards = page.locator('[data-testid="disk-card"]');
    const diskCount = await diskCards.count();
    expect(diskCount).toBeGreaterThan(0);

    // First disk should show required information
    const firstDisk = diskCards.first();
    await expect(firstDisk.locator('[data-testid="disk-mount-point"]')).toBeVisible();
    await expect(firstDisk.locator('[data-testid="disk-total-space"]')).toBeVisible();
    await expect(firstDisk.locator('[data-testid="disk-free-space"]')).toBeVisible();
    await expect(firstDisk.locator('[data-testid="disk-usage-percent"]')).toBeVisible();
  });

  test('should display disk usage with visual indicator', async ({ page }) => {
    // RED PHASE: Test disk usage visualization

    await page.waitForSelector('[data-testid="disk-card"]');
    const firstDisk = page.locator('[data-testid="disk-card"]').first();

    // Should have a progress bar or visual indicator
    const usageBar = firstDisk.locator('[data-testid="disk-usage-bar"]');
    await expect(usageBar).toBeVisible();

    // Usage percentage should be displayed
    const usageText = await firstDisk.locator('[data-testid="disk-usage-percent"]').textContent();
    expect(usageText).toMatch(/\d+(\.\d+)?%/); // Match percentage format
  });

  test('should allow selecting a disk for recordings', async ({ page }) => {
    // RED PHASE: Test disk selection interaction

    await page.waitForSelector('[data-testid="disk-card"]');

    // Click on the first disk to select it
    const firstDisk = page.locator('[data-testid="disk-card"]').first();
    await firstDisk.click();

    // Should show as selected (visual feedback)
    await expect(firstDisk).toHaveClass(/selected|active|checked/i);

    // Should enable the "Configure Volume" button
    const configureButton = page.locator('[data-testid="configure-volume-button"]');
    await expect(configureButton).toBeEnabled();
  });

  test('should open volume configuration dialog', async ({ page }) => {
    // RED PHASE: Test volume configuration dialog

    await page.waitForSelector('[data-testid="disk-card"]');

    // Select disk and open configuration
    await page.locator('[data-testid="disk-card"]').first().click();
    await page.locator('[data-testid="configure-volume-button"]').click();

    // Should show volume configuration dialog
    const dialog = page.locator('[data-testid="volume-config-dialog"]');
    await expect(dialog).toBeVisible();

    // Dialog should have required fields
    await expect(page.locator('[data-testid="host-path-input"]')).toBeVisible();
    await expect(page.locator('[data-testid="container-path-input"]')).toBeVisible();
  });

  test('should validate volume path input', async ({ page }) => {
    // RED PHASE: Test path validation

    await page.waitForSelector('[data-testid="disk-card"]');
    await page.locator('[data-testid="disk-card"]').first().click();
    await page.locator('[data-testid="configure-volume-button"]').click();

    const hostPathInput = page.locator('[data-testid="host-path-input"]');
    const saveButton = page.locator('[data-testid="save-volume-button"]');

    // Try invalid relative path
    await hostPathInput.fill('./relative/path');
    await saveButton.click();

    // Should show validation error
    const errorMessage = page.locator('[data-testid="path-validation-error"]');
    await expect(errorMessage).toBeVisible();
    await expect(errorMessage).toContainText(/absolute|invalid/i);
  });

  test('should accept valid absolute path', async ({ page }) => {
    // RED PHASE: Test valid path acceptance

    await page.waitForSelector('[data-testid="disk-card"]');
    await page.locator('[data-testid="disk-card"]').first().click();
    await page.locator('[data-testid="configure-volume-button"]').click();

    const hostPathInput = page.locator('[data-testid="host-path-input"]');
    const containerPathInput = page.locator('[data-testid="container-path-input"]');
    const saveButton = page.locator('[data-testid="save-volume-button"]');

    // Enter valid paths
    await hostPathInput.fill('/mnt/storage/frigate');
    await containerPathInput.fill('/media/frigate/recordings');
    await saveButton.click();

    // Dialog should close (no validation errors)
    const dialog = page.locator('[data-testid="volume-config-dialog"]');
    await expect(dialog).not.toBeVisible({ timeout: 2000 });
  });

  test('should display configured volumes in list', async ({ page }) => {
    // RED PHASE: Test volume list display after configuration

    // Configure a volume
    await configureVolume(page, '/mnt/storage/frigate', '/media/frigate/recordings');

    // Should display in configured volumes list
    const volumeList = page.locator('[data-testid="configured-volumes-list"]');
    await expect(volumeList).toBeVisible();

    const volumeItem = volumeList.locator('[data-testid="volume-item"]').first();
    await expect(volumeItem).toContainText('/mnt/storage/frigate');
    await expect(volumeItem).toContainText('/media/frigate/recordings');
  });

  test('should allow editing configured volume', async ({ page }) => {
    // RED PHASE: Test volume editing

    // Configure a volume
    await configureVolume(page, '/mnt/storage/frigate', '/media/frigate/recordings');

    // Click edit button
    const editButton = page.locator('[data-testid="volume-item"]')
      .first()
      .locator('[data-testid="edit-volume-button"]');
    await editButton.click();

    // Should reopen dialog with existing values
    const dialog = page.locator('[data-testid="volume-config-dialog"]');
    await expect(dialog).toBeVisible();

    const hostPathInput = page.locator('[data-testid="host-path-input"]');
    await expect(hostPathInput).toHaveValue('/mnt/storage/frigate');

    // Modify the path
    await hostPathInput.fill('/mnt/storage2/frigate');
    await page.locator('[data-testid="save-volume-button"]').click();

    // Should update the volume in the list
    await expect(page.locator('[data-testid="volume-item"]').first())
      .toContainText('/mnt/storage2/frigate');
  });

  test('should allow removing configured volume', async ({ page }) => {
    // RED PHASE: Test volume removal

    // Configure a volume
    await configureVolume(page, '/mnt/storage/frigate', '/media/frigate/recordings');

    // Get initial count
    const volumeList = page.locator('[data-testid="configured-volumes-list"]');
    const initialCount = await volumeList.locator('[data-testid="volume-item"]').count();

    // Click remove button
    const removeButton = page.locator('[data-testid="volume-item"]')
      .first()
      .locator('[data-testid="remove-volume-button"]');
    await removeButton.click();

    // Should show confirmation dialog
    const confirmDialog = page.locator('[data-testid="confirm-remove-dialog"]');
    await expect(confirmDialog).toBeVisible();

    await page.locator('[data-testid="confirm-remove-yes"]').click();

    // Volume should be removed from list
    const newCount = await volumeList.locator('[data-testid="volume-item"]').count();
    expect(newCount).toBe(initialCount - 1);
  });

  test('should show Docker Run command preview', async ({ page }) => {
    // RED PHASE: Test Docker Run command generation

    // Configure a volume
    await configureVolume(page, '/mnt/storage/frigate', '/media/frigate/recordings');

    // Switch to Docker Run preview
    await page.locator('[data-testid="preview-mode-docker-run"]').click();

    // Should show docker run command
    const preview = page.locator('[data-testid="docker-run-preview"]');
    await expect(preview).toBeVisible();

    const commandText = await preview.textContent();
    expect(commandText).toContain('docker run');
    expect(commandText).toContain('-v');
    expect(commandText).toContain('/mnt/storage/frigate:/media/frigate/recordings');
  });

  test('should show Docker Compose YAML preview', async ({ page }) => {
    // RED PHASE: Test Docker Compose YAML generation

    // Configure a volume
    await configureVolume(page, '/mnt/storage/frigate', '/media/frigate/recordings');

    // Switch to Docker Compose preview
    await page.locator('[data-testid="preview-mode-docker-compose"]').click();

    // Should show docker-compose.yml
    const preview = page.locator('[data-testid="docker-compose-preview"]');
    await expect(preview).toBeVisible();

    const yamlText = await preview.textContent();
    expect(yamlText).toContain('volumes:');
    expect(yamlText).toContain('/mnt/storage/frigate:/media/frigate/recordings');
  });

  test('should copy command to clipboard', async ({ page }) => {
    // RED PHASE: Test copy-to-clipboard functionality

    // Configure a volume
    await configureVolume(page, '/mnt/storage/frigate', '/media/frigate/recordings');

    // Click copy button
    const copyButton = page.locator('[data-testid="copy-command-button"]');
    await copyButton.click();

    // Should show success message
    const successMessage = page.locator('[data-testid="copy-success-message"]');
    await expect(successMessage).toBeVisible();
    await expect(successMessage).toContainText(/copied|复制成功/i);
  });

  test('should warn about low disk space', async ({ page }) => {
    // RED PHASE: Test low disk space warning

    // Mock a disk with low free space
    await page.route('**/api/disks', async (route) => {
      await route.fulfill({
        status: 200,
        body: JSON.stringify([
          {
            mount_point: '/mnt/low-space',
            total: 100 * 1024 * 1024 * 1024, // 100 GB
            free: 3 * 1024 * 1024 * 1024,    // 3 GB (below 10GB threshold)
            used: 97 * 1024 * 1024 * 1024,
            usage_percent: 97.0,
            total_formatted: '100 GB',
            free_formatted: '3 GB',
            used_formatted: '97 GB',
          },
        ]),
      });
    });

    await page.reload();
    await page.waitForSelector('[data-testid="disk-card"]');

    // Should show warning badge or indicator
    const warningIndicator = page.locator('[data-testid="low-space-warning"]');
    await expect(warningIndicator).toBeVisible();
    await expect(warningIndicator).toContainText(/low|不足/i);
  });

  test('should handle cross-platform path formats', async ({ page }) => {
    // RED PHASE: Test platform-specific path handling

    // Mock platform detection
    const platform = process.platform;

    await page.waitForSelector('[data-testid="disk-card"]');
    await page.locator('[data-testid="disk-card"]').first().click();
    await page.locator('[data-testid="configure-volume-button"]').click();

    const hostPathInput = page.locator('[data-testid="host-path-input"]');

    if (platform === 'win32') {
      // Test Windows path
      await hostPathInput.fill('C:\\ProgramData\\Frigate');

      const pathDisplay = page.locator('[data-testid="formatted-path-display"]');
      await expect(pathDisplay).toContainText('C:\\ProgramData\\Frigate');
    } else {
      // Test Unix path
      await hostPathInput.fill('/var/lib/frigate');

      const pathDisplay = page.locator('[data-testid="formatted-path-display"]');
      await expect(pathDisplay).toContainText('/var/lib/frigate');
    }
  });

  test('should persist configuration on page reload', async ({ page }) => {
    // RED PHASE: Test configuration persistence

    // Configure a volume
    await configureVolume(page, '/mnt/storage/frigate', '/media/frigate/recordings');

    // Reload the page
    await page.reload();
    await page.waitForSelector('[data-testid="configured-volumes-list"]');

    // Should still show configured volume
    const volumeItem = page.locator('[data-testid="volume-item"]').first();
    await expect(volumeItem).toContainText('/mnt/storage/frigate');
    await expect(volumeItem).toContainText('/media/frigate/recordings');
  });

  test('should show error message when disk detection fails', async ({ page }) => {
    // RED PHASE: Test error handling

    // Mock disk detection failure
    await page.route('**/api/disks', async (route) => {
      await route.abort('failed');
    });

    await page.reload();

    // Should show error message
    const errorMessage = page.locator('[data-testid="disk-detection-error"]');
    await expect(errorMessage).toBeVisible();
    await expect(errorMessage).toContainText(/error|failed|错误/i);

    // Should show retry button
    const retryButton = page.locator('[data-testid="retry-detection-button"]');
    await expect(retryButton).toBeVisible();
  });

  test('should navigate to next step after configuration', async ({ page }) => {
    // RED PHASE: Test navigation flow

    // Configure a volume
    await configureVolume(page, '/mnt/storage/frigate', '/media/frigate/recordings');

    // Click "Next" or "Continue" button
    const nextButton = page.locator('[data-testid="next-step-button"]');
    await expect(nextButton).toBeEnabled();
    await nextButton.click();

    // Should navigate to next page (e.g., deployment review)
    await expect(page).toHaveURL(/deployment|review/i);
  });
});

// Helper function to configure a volume
async function configureVolume(page: Page, hostPath: string, containerPath: string) {
  await page.waitForSelector('[data-testid="disk-card"]');
  await page.locator('[data-testid="disk-card"]').first().click();
  await page.locator('[data-testid="configure-volume-button"]').click();

  const hostPathInput = page.locator('[data-testid="host-path-input"]');
  const containerPathInput = page.locator('[data-testid="container-path-input"]');
  const saveButton = page.locator('[data-testid="save-volume-button"]');

  await hostPathInput.fill(hostPath);
  await containerPathInput.fill(containerPath);
  await saveButton.click();

  // Wait for dialog to close
  await page.waitForSelector('[data-testid="volume-config-dialog"]', { state: 'hidden' });
}
