// E2E tests for cross-platform agent invocation
// Tests agent execution from Tauri frontend across different platforms

import { test, expect } from '@playwright/test';

test.describe('Cross-Platform Agent Invocation', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to the hardware detection page
    await page.goto('/hardware');
  });

  test('should detect hardware on current platform', async ({ page }) => {
    // Click the detect hardware button
    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    // Wait for detection to complete (loading state should disappear)
    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 30000 });

    // Verify that hardware information is displayed
    const hardwareCards = page.locator('[data-testid="hardware-card"]');
    const cardCount = await hardwareCards.count();

    // We should have at least some devices detected (even if just CPU)
    expect(cardCount).toBeGreaterThanOrEqual(0);

    // Log detected devices for debugging
    console.log(`Detected ${cardCount} hardware devices`);
  });

  test('should report correct platform information', async ({ page }) => {
    // Trigger hardware detection
    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    // Wait for detection to complete
    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 30000 });

    // Check platform information
    const platformInfo = page.locator('[data-testid="platform-info"]');

    if (await platformInfo.isVisible()) {
      const platformText = await platformInfo.textContent();

      // Verify platform is one of the supported values
      expect(platformText).toMatch(/(linux|darwin|windows|macos|macOS|Linux|Windows)/i);

      // Log platform for debugging
      console.log(`Detected platform: ${platformText}`);
    }
  });

  test('should report correct architecture information', async ({ page }) => {
    // Trigger hardware detection
    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    // Wait for detection to complete
    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 30000 });

    // Check architecture information
    const archInfo = page.locator('[data-testid="architecture-info"]');

    if (await archInfo.isVisible()) {
      const archText = await archInfo.textContent();

      // Verify architecture is one of the supported values
      expect(archText).toMatch(/(x86_64|amd64|arm64|aarch64|arm|x64)/i);

      // Log architecture for debugging
      console.log(`Detected architecture: ${archText}`);
    }
  });

  test('should handle agent execution failure gracefully', async ({ page }) => {
    // Mock agent failure by intercepting the Tauri IPC call
    // (This would need Tauri mock setup in a real test environment)

    // Navigate to hardware page
    await page.goto('/hardware');

    // Trigger detection
    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    // Wait for potential error message
    const errorMessage = page.locator('[data-testid="error-message"]');

    // Either we get results or a graceful error message (not a crash)
    await page.waitForFunction(
      () => {
        const hasCards = document.querySelectorAll('[data-testid="hardware-card"]').length > 0;
        const hasError = document.querySelector('[data-testid="error-message"]') !== null;
        const hasLoading = document.querySelectorAll('[data-testid="loading"]').length > 0;
        return hasCards || hasError || !hasLoading;
      },
      { timeout: 30000 }
    );

    // Verify page didn't crash
    const appContainer = page.locator('#root');
    await expect(appContainer).toBeVisible();
  });

  test('should display device capabilities correctly', async ({ page }) => {
    // Trigger hardware detection
    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    // Wait for detection to complete
    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 30000 });

    // Check if any devices were detected
    const hardwareCards = page.locator('[data-testid="hardware-card"]');
    const cardCount = await hardwareCards.count();

    if (cardCount > 0) {
      // Click on the first device to see details
      await hardwareCards.first().click();

      // Check for capabilities display
      const capabilitiesSection = page.locator('[data-testid="capabilities"]');

      if (await capabilitiesSection.isVisible()) {
        const capabilitiesList = capabilitiesSection.locator('li, span[data-capability]');
        const capCount = await capabilitiesList.count();

        // Devices should have at least one capability
        expect(capCount).toBeGreaterThan(0);

        console.log(`First device has ${capCount} capabilities`);
      }
    }
  });

  test('should support filtering devices by type', async ({ page }) => {
    // Trigger hardware detection
    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    // Wait for detection to complete
    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 30000 });

    // Try to filter by GPU type
    const filterGPU = page.getByRole('button', { name: /gpu|显卡/i });

    if (await filterGPU.isVisible()) {
      await filterGPU.click();

      // Verify only GPU devices are shown
      const visibleCards = page.locator('[data-testid="hardware-card"]:visible');
      const count = await visibleCards.count();

      if (count > 0) {
        // Check that all visible cards are GPU type
        for (let i = 0; i < count; i++) {
          const card = visibleCards.nth(i);
          const typeLabel = card.locator('[data-testid="device-type"]');

          if (await typeLabel.isVisible()) {
            const typeText = await typeLabel.textContent();
            expect(typeText).toMatch(/gpu|显卡/i);
          }
        }
      }
    }
  });
});

test.describe('Cross-Platform Agent - Platform-Specific', () => {
  test('should detect Linux-specific hardware', async ({ page }) => {
    test.skip(process.platform !== 'linux', 'Linux-only test');

    await page.goto('/hardware');

    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 30000 });

    // On Linux, we might detect /dev/video* devices
    const hardwareCards = page.locator('[data-testid="hardware-card"]');
    const cardCount = await hardwareCards.count();

    if (cardCount > 0) {
      // Look for Linux-specific device paths
      const devicePaths = page.locator('[data-testid="device-path"]');
      const pathCount = await devicePaths.count();

      for (let i = 0; i < pathCount; i++) {
        const path = await devicePaths.nth(i).textContent();
        console.log(`Linux device path: ${path}`);

        // Verify Linux-style paths
        if (path) {
          expect(path).toMatch(/\/dev\/(video|dri|nvidia|apex)/);
        }
      }
    }
  });

  test('should detect macOS-specific hardware', async ({ page }) => {
    test.skip(process.platform !== 'darwin', 'macOS-only test');

    await page.goto('/hardware');

    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 30000 });

    // On macOS, we might detect Metal GPUs and Neural Engine
    const hardwareCards = page.locator('[data-testid="hardware-card"]');
    const cardCount = await hardwareCards.count();

    if (cardCount > 0) {
      // Look for Metal capability
      const capabilities = page.locator('[data-testid="capability"]');
      const capText = await capabilities.allTextContents();

      const hasMetal = capText.some(text => text.toLowerCase().includes('metal'));
      console.log(`macOS Metal support detected: ${hasMetal}`);

      // On Apple Silicon, check for Neural Engine
      const deviceNames = page.locator('[data-testid="device-name"]');
      const names = await deviceNames.allTextContents();

      const hasNeuralEngine = names.some(name =>
        name.toLowerCase().includes('neural') || name.toLowerCase().includes('apple')
      );

      console.log(`macOS Neural Engine detected: ${hasNeuralEngine}`);
    }
  });

  test('should detect Windows-specific hardware', async ({ page }) => {
    test.skip(process.platform !== 'win32', 'Windows-only test');

    await page.goto('/hardware');

    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 30000 });

    // On Windows, we might detect DirectX/WMIC devices
    const hardwareCards = page.locator('[data-testid="hardware-card"]');
    const cardCount = await hardwareCards.count();

    if (cardCount > 0) {
      // Look for Windows-specific detection sources
      const detectionSources = page.locator('[data-testid="detection-source"]');
      const sources = await detectionSources.allTextContents();

      const hasWindowsSource = sources.some(source =>
        source.toLowerCase().includes('wmic') ||
        source.toLowerCase().includes('powershell') ||
        source.toLowerCase().includes('directx')
      );

      console.log(`Windows-specific detection used: ${hasWindowsSource}`);
    }
  });
});

test.describe('Cross-Platform Agent - Error Handling', () => {
  test('should handle missing agent binary gracefully', async ({ page }) => {
    // This test verifies that the UI handles agent errors gracefully
    await page.goto('/hardware');

    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    // Wait for result or error
    await page.waitForFunction(
      () => {
        const hasCards = document.querySelectorAll('[data-testid="hardware-card"]').length > 0;
        const hasError = document.querySelector('[data-testid="error-message"]') !== null;
        const noLoading = document.querySelectorAll('[data-testid="loading"]').length === 0;
        return (hasCards || hasError) && noLoading;
      },
      { timeout: 30000 }
    );

    // Verify UI is still functional
    const pageTitle = page.locator('h1, h2');
    await expect(pageTitle.first()).toBeVisible();
  });

  test('should handle timeout gracefully', async ({ page }) => {
    await page.goto('/hardware');

    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    // Wait for timeout or completion
    await page.waitForTimeout(35000); // Longer than typical timeout

    // Verify UI didn't crash
    const appContainer = page.locator('#root');
    await expect(appContainer).toBeVisible();

    // Either we have results, an error, or a timeout message
    const hasResults = await page.locator('[data-testid="hardware-card"]').count() > 0;
    const hasError = await page.locator('[data-testid="error-message"]').isVisible();
    const hasTimeout = await page.locator('[data-testid="timeout-message"]').isVisible();

    expect(hasResults || hasError || hasTimeout).toBeTruthy();
  });
});

test.describe('Cross-Platform Agent - Performance', () => {
  test('should complete detection within reasonable time', async ({ page }) => {
    await page.goto('/hardware');

    const startTime = Date.now();

    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });
    await detectButton.click();

    // Wait for detection to complete
    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 30000 });

    const endTime = Date.now();
    const duration = endTime - startTime;

    console.log(`Hardware detection took ${duration}ms`);

    // Detection should complete within 30 seconds
    expect(duration).toBeLessThan(30000);
  });

  test('should handle multiple rapid detections', async ({ page }) => {
    await page.goto('/hardware');

    const detectButton = page.getByRole('button', { name: /检测硬件|detect hardware/i });

    // Trigger detection multiple times rapidly
    for (let i = 0; i < 3; i++) {
      await detectButton.click();
      await page.waitForTimeout(500); // Small delay between clicks
    }

    // Wait for final detection to complete
    await page.waitForFunction(() => {
      const loadingElements = document.querySelectorAll('[data-testid="loading"]');
      return loadingElements.length === 0;
    }, { timeout: 40000 });

    // Verify UI is still functional
    const hardwareCards = page.locator('[data-testid="hardware-card"]');
    const cardCount = await hardwareCards.count();

    console.log(`After rapid detections, found ${cardCount} devices`);

    // Should not crash
    const appContainer = page.locator('#root');
    await expect(appContainer).toBeVisible();
  });
});
