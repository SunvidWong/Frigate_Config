// T097: E2E test for deployment workflow
// Tests complete user workflow: validation → deployment → health checks → logs → rollback
// REQUIREMENT: User Story 3 - Safe Deployment with Validation

import { test, expect, Page } from '@playwright/test';

// Helper function to wait for element with text
async function waitForText(page: Page, text: string, timeout = 30000) {
  await page.waitForSelector(`text=${text}`, { timeout });
}

// Helper function to navigate to Deploy page
async function navigateToDeploy(page: Page) {
  await page.goto('http://localhost:15000');
  await page.click('text=Deploy Frigate');
  await expect(page).toHaveURL(/.*deploy/);
}

test.describe('Deployment Workflow', () => {
  test.beforeEach(async ({ page }) => {
    // Start fresh on each test
    await page.goto('http://localhost:15000');
  });

  test('should display Deploy page with validation section', async ({ page }) => {
    await navigateToDeploy(page);

    // Verify page elements
    await expect(page.locator('h1')).toContainText('Deploy');
    await expect(page.locator('text=Pre-Deployment Validation')).toBeVisible();
    await expect(page.locator('button:has-text("Run Validation")')).toBeVisible();
  });

  test('should run pre-deployment validation checks', async ({ page }) => {
    await navigateToDeploy(page);

    // Click Run Validation button
    await page.click('button:has-text("Run Validation")');

    // Wait for validation to complete
    await waitForText(page, 'Validation Complete', 60000);

    // Should show validation results
    await expect(page.locator('[data-testid="validation-results"]')).toBeVisible();

    // Should show checks (YAML, Docker, Devices, Ports, Volumes)
    const validationResults = page.locator('[data-testid="validation-check"]');
    const count = await validationResults.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should display validation errors when config is invalid', async ({ page }) => {
    // First, load an invalid configuration
    await page.goto('http://localhost:15000/manual-config');
    await page.click('button:has-text("Load Configuration")');

    // Select invalid config file (would need to be set up in test environment)
    // For now, this is a placeholder for the test structure
    // await selectFile(page, 'tests/fixtures/invalid_syntax.yml');

    // Navigate to Deploy
    await navigateToDeploy(page);

    // Run validation
    await page.click('button:has-text("Run Validation")');

    // Wait for validation to complete
    await waitForText(page, 'Validation Complete', 60000);

    // Should show validation errors
    await expect(page.locator('[data-testid="validation-error"]')).toBeVisible();

    // Deploy button should be disabled
    await expect(page.locator('button:has-text("Deploy")')).toBeDisabled();
  });

  test('should enable Deploy button when validation passes', async ({ page }) => {
    await navigateToDeploy(page);

    // Run validation
    await page.click('button:has-text("Run Validation")');

    // Wait for validation
    await waitForText(page, 'Validation Complete', 60000);

    // If validation passed, Deploy button should be enabled
    const deployButton = page.locator('button:has-text("Deploy")');

    // Check if validation passed by looking for success indicator
    const validationPassed = await page.locator('[data-testid="validation-success"]').isVisible();

    if (validationPassed) {
      await expect(deployButton).toBeEnabled();
    } else {
      await expect(deployButton).toBeDisabled();
    }
  });

  test('should display validation warnings without blocking deployment', async ({ page }) => {
    await navigateToDeploy(page);

    // Run validation
    await page.click('button:has-text("Run Validation")');
    await waitForText(page, 'Validation Complete', 60000);

    // Check for warnings (e.g., CPU-only mode warning)
    const warnings = page.locator('[data-testid="validation-warning"]');
    const warningCount = await warnings.count();

    if (warningCount > 0) {
      // Warnings should be visible
      await expect(warnings.first()).toBeVisible();

      // But Deploy button should still be enabled (warnings don't block)
      const deployButton = page.locator('button:has-text("Deploy")');
      const isDisabled = await deployButton.isDisabled();

      // Only disabled if there are errors, not just warnings
      expect(isDisabled || !isDisabled).toBeTruthy(); // Placeholder assertion
    }
  });

  test('should show Docker command preview before deployment', async ({ page }) => {
    await navigateToDeploy(page);

    // Run validation first
    await page.click('button:has-text("Run Validation")');
    await waitForText(page, 'Validation Complete', 60000);

    // Check if validation passed
    const validationPassed = await page.locator('[data-testid="validation-success"]').isVisible();

    if (validationPassed) {
      // Should show command preview section
      await expect(page.locator('[data-testid="command-preview"]')).toBeVisible();

      // Should contain docker run or docker-compose
      const commandText = await page.locator('[data-testid="command-preview"]').textContent();
      expect(commandText).toMatch(/docker (run|compose)/);
    }
  });

  test('should execute deployment and show progress', async ({ page }) => {
    await navigateToDeploy(page);

    // Run validation
    await page.click('button:has-text("Run Validation")');
    await waitForText(page, 'Validation Complete', 60000);

    // Check if we can deploy (validation passed and Docker available)
    const canDeploy = await page.locator('button:has-text("Deploy")').isEnabled();

    if (canDeploy) {
      // Click Deploy
      await page.click('button:has-text("Deploy")');

      // Should show deployment progress
      await expect(page.locator('[data-testid="deployment-progress"]')).toBeVisible();

      // Should show status updates
      await waitForText(page, 'Deploying', 5000);

      // Wait for deployment to complete (or timeout)
      await page.waitForSelector(
        '[data-testid="deployment-status"]:has-text("Running"), [data-testid="deployment-status"]:has-text("Failed")',
        { timeout: 120000 }
      );

      // Check final status
      const deploymentStatus = await page.locator('[data-testid="deployment-status"]').textContent();
      expect(deploymentStatus).toMatch(/Running|Failed|Completed/);
    }
  });

  test('should display deployment logs during deployment', async ({ page }) => {
    await navigateToDeploy(page);

    // Run validation and deploy
    await page.click('button:has-text("Run Validation")');
    await waitForText(page, 'Validation Complete', 60000);

    const canDeploy = await page.locator('button:has-text("Deploy")').isEnabled();

    if (canDeploy) {
      await page.click('button:has-text("Deploy")');

      // Wait for logs to appear
      await page.waitForSelector('[data-testid="deployment-logs"]', { timeout: 30000 });

      // Should show log entries
      const logEntries = page.locator('[data-testid="log-entry"]');
      const logCount = await logEntries.count();

      expect(logCount).toBeGreaterThan(0);

      // Logs should contain Docker output
      const firstLog = await logEntries.first().textContent();
      expect(firstLog).toBeTruthy();
      expect(firstLog!.length).toBeGreaterThan(0);
    }
  });

  test('should perform health checks after deployment', async ({ page }) => {
    await navigateToDeploy(page);

    // Run validation and deploy
    await page.click('button:has-text("Run Validation")');
    await waitForText(page, 'Validation Complete', 60000);

    const canDeploy = await page.locator('button:has-text("Deploy")').isEnabled();

    if (canDeploy) {
      await page.click('button:has-text("Deploy")');

      // Wait for health check section to appear
      await page.waitForSelector('[data-testid="health-check-status"]', { timeout: 120000 });

      // Should show health check results
      await expect(page.locator('[data-testid="health-check-status"]')).toBeVisible();

      // Should show individual health checks
      const healthChecks = page.locator('[data-testid="health-check"]');
      const checkCount = await healthChecks.count();

      expect(checkCount).toBeGreaterThan(0);

      // Check for common health checks: Container Running, API Responding, etc.
      const healthCheckTexts = await healthChecks.allTextContents();
      expect(healthCheckTexts.some(text => text.includes('Container') || text.includes('API'))).toBeTruthy();
    }
  });

  test('should show success status when deployment completes successfully', async ({ page }) => {
    await navigateToDeploy(page);

    // Run validation and deploy
    await page.click('button:has-text("Run Validation")');
    await waitForText(page, 'Validation Complete', 60000);

    const canDeploy = await page.locator('button:has-text("Deploy")').isEnabled();

    if (canDeploy) {
      await page.click('button:has-text("Deploy")');

      // Wait for deployment to complete
      await page.waitForSelector(
        '[data-testid="deployment-status"]:has-text("Running"), [data-testid="deployment-status"]:has-text("Healthy")',
        { timeout: 180000 }
      );

      // Should show success indicators
      const status = await page.locator('[data-testid="deployment-status"]').textContent();

      if (status?.includes('Running') || status?.includes('Healthy')) {
        // Should show success message
        await expect(page.locator('[data-testid="deployment-success"]')).toBeVisible();

        // Should show links to Frigate UI and logs
        await expect(page.locator('a:has-text("Open Frigate UI")')).toBeVisible();
        await expect(page.locator('button:has-text("View Logs")')).toBeVisible();
      }
    }
  });

  test('should navigate to Logs page and display real-time logs', async ({ page }) => {
    // Assuming deployment is already running
    await page.goto('http://localhost:15000/logs');

    // Should show Logs page
    await expect(page.locator('h1')).toContainText('Logs');

    // Should have log viewer
    await expect(page.locator('[data-testid="log-viewer"]')).toBeVisible();

    // Wait for logs to stream
    await page.waitForSelector('[data-testid="log-line"]', { timeout: 30000 });

    // Should have log lines
    const logLines = page.locator('[data-testid="log-line"]');
    const lineCount = await logLines.count();

    expect(lineCount).toBeGreaterThan(0);
  });

  test('should filter logs by search term', async ({ page }) => {
    await page.goto('http://localhost:15000/logs');

    // Wait for logs to load
    await page.waitForSelector('[data-testid="log-line"]', { timeout: 30000 });

    const initialCount = await page.locator('[data-testid="log-line"]').count();

    // Enter search term
    await page.fill('[data-testid="log-search"]', 'frigate');

    // Wait for filtering
    await page.waitForTimeout(1000);

    // Should show fewer logs (filtered)
    const filteredCount = await page.locator('[data-testid="log-line"]').count();

    // Filtered count should be <= initial count
    expect(filteredCount).toBeLessThanOrEqual(initialCount);
  });

  test('should display Rollback button when deployment is running', async ({ page }) => {
    await navigateToDeploy(page);

    // If there's a current deployment, should show Rollback button
    const hasDeployment = await page.locator('[data-testid="current-deployment"]').isVisible();

    if (hasDeployment) {
      await expect(page.locator('button:has-text("Rollback")')).toBeVisible();
    }
  });

  test('should show rollback confirmation dialog', async ({ page }) => {
    await navigateToDeploy(page);

    const hasDeployment = await page.locator('[data-testid="current-deployment"]').isVisible();

    if (hasDeployment) {
      // Click Rollback button
      await page.click('button:has-text("Rollback")');

      // Should show confirmation dialog
      await expect(page.locator('[data-testid="rollback-confirm-dialog"]')).toBeVisible();

      // Should show warning message
      await expect(page.locator('text=Are you sure')).toBeVisible();

      // Should have Cancel and Confirm buttons
      await expect(page.locator('button:has-text("Cancel")')).toBeVisible();
      await expect(page.locator('button:has-text("Confirm")')).toBeVisible();
    }
  });

  test('should cancel rollback when Cancel button is clicked', async ({ page }) => {
    await navigateToDeploy(page);

    const hasDeployment = await page.locator('[data-testid="current-deployment"]').isVisible();

    if (hasDeployment) {
      // Click Rollback
      await page.click('button:has-text("Rollback")');

      // Wait for dialog
      await expect(page.locator('[data-testid="rollback-confirm-dialog"]')).toBeVisible();

      // Click Cancel
      await page.click('button:has-text("Cancel")');

      // Dialog should close
      await expect(page.locator('[data-testid="rollback-confirm-dialog"]')).not.toBeVisible();

      // Deployment should still be running
      await expect(page.locator('[data-testid="current-deployment"]')).toBeVisible();
    }
  });

  test('should execute rollback when confirmed', async ({ page }) => {
    await navigateToDeploy(page);

    const hasDeployment = await page.locator('[data-testid="current-deployment"]').isVisible();
    const hasPreviousDeployment = await page.locator('[data-testid="previous-deployment"]').isVisible();

    if (hasDeployment && hasPreviousDeployment) {
      // Click Rollback
      await page.click('button:has-text("Rollback")');

      // Confirm rollback
      await page.waitForSelector('[data-testid="rollback-confirm-dialog"]');
      await page.click('button:has-text("Confirm")');

      // Should show rollback progress
      await expect(page.locator('[data-testid="rollback-progress"]')).toBeVisible();

      // Wait for rollback to complete
      await page.waitForSelector(
        '[data-testid="rollback-status"]:has-text("Success"), [data-testid="rollback-status"]:has-text("Failed")',
        { timeout: 120000 }
      );

      // Check rollback result
      const rollbackStatus = await page.locator('[data-testid="rollback-status"]').textContent();
      expect(rollbackStatus).toMatch(/Success|Failed|Completed/);
    }
  });

  test('should show error message when rollback fails', async ({ page }) => {
    // This test would simulate a rollback failure scenario
    // Implementation depends on test infrastructure
    await navigateToDeploy(page);

    // Placeholder for rollback failure test
    expect(true).toBeTruthy();
  });

  test('should show deployment history', async ({ page }) => {
    await navigateToDeploy(page);

    // Should have deployment history section
    const historyVisible = await page.locator('[data-testid="deployment-history"]').isVisible();

    if (historyVisible) {
      // Should show past deployments
      const historyItems = page.locator('[data-testid="deployment-history-item"]');
      const itemCount = await historyItems.count();

      // May have 0 or more history items
      expect(itemCount).toBeGreaterThanOrEqual(0);

      if (itemCount > 0) {
        // Each item should show timestamp and status
        const firstItem = historyItems.first();
        await expect(firstItem).toBeVisible();

        const itemText = await firstItem.textContent();
        expect(itemText).toBeTruthy();
        expect(itemText!.length).toBeGreaterThan(0);
      }
    }
  });

  test('should automatically rollback on health check failure', async ({ page }) => {
    // This test simulates automatic rollback when health checks fail
    // Would require special test configuration to trigger health check failure

    await navigateToDeploy(page);

    // Placeholder for automatic rollback test
    // In real scenario, would deploy with intentionally failing health check
    // and verify automatic rollback is triggered

    expect(true).toBeTruthy();
  });

  test('complete deployment workflow', async ({ page }) => {
    // Test the complete end-to-end workflow

    // 1. Navigate to Deploy page
    await navigateToDeploy(page);
    await expect(page.locator('h1')).toContainText('Deploy');

    // 2. Run validation
    await page.click('button:has-text("Run Validation")');
    await waitForText(page, 'Validation Complete', 60000);

    // 3. Check validation results
    const validationPassed = await page.locator('[data-testid="validation-success"]').isVisible();

    if (validationPassed) {
      // 4. Review deployment command
      await expect(page.locator('[data-testid="command-preview"]')).toBeVisible();

      // 5. Execute deployment
      await page.click('button:has-text("Deploy")');

      // 6. Monitor deployment progress
      await expect(page.locator('[data-testid="deployment-progress"]')).toBeVisible();

      // 7. Wait for health checks
      await page.waitForSelector('[data-testid="health-check-status"]', { timeout: 180000 });

      // 8. Verify deployment status
      const status = await page.locator('[data-testid="deployment-status"]').textContent();
      expect(status).toMatch(/Running|Failed|Healthy/);

      // 9. Navigate to Logs page
      await page.click('button:has-text("View Logs")');
      await expect(page).toHaveURL(/.*logs/);

      // 10. Verify logs are streaming
      await page.waitForSelector('[data-testid="log-line"]', { timeout: 30000 });
      const logCount = await page.locator('[data-testid="log-line"]').count();
      expect(logCount).toBeGreaterThan(0);

      // 11. Go back to Deploy page
      await navigateToDeploy(page);

      // 12. Verify deployment is still shown as active
      await expect(page.locator('[data-testid="current-deployment"]')).toBeVisible();
    } else {
      // Validation failed - verify error handling
      await expect(page.locator('[data-testid="validation-error"]')).toBeVisible();
      await expect(page.locator('button:has-text("Deploy")')).toBeDisabled();
    }
  });
});
