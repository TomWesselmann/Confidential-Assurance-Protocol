/**
 * Base Page Object for CAP Desktop App
 *
 * @description Common page interactions and selectors
 */

export class BasePage {
  /**
   * Wait for app to be ready
   */
  async waitForAppReady(): Promise<void> {
    await browser.waitUntil(
      async () => {
        const body = await $('body');
        return body.isExisting();
      },
      {
        timeout: 30000,
        timeoutMsg: 'App did not load within 30 seconds',
      }
    );
  }

  /**
   * Get element by data-testid attribute
   */
  async getByTestId(testId: string): Promise<WebdriverIO.Element> {
    return $(`[data-testid="${testId}"]`);
  }

  /**
   * Wait for element to be visible
   */
  async waitForElement(selector: string, timeout = 10000): Promise<WebdriverIO.Element> {
    const element = await $(selector);
    await element.waitForDisplayed({ timeout });
    return element;
  }

  /**
   * Click a button by text content
   */
  async clickButton(text: string): Promise<void> {
    const button = await $(`button*=${text}`);
    await button.waitForClickable({ timeout: 5000 });
    await button.click();
  }

  /**
   * Check if toast/notification is visible
   */
  async isToastVisible(message?: string): Promise<boolean> {
    const toast = await $('[role="alert"]');
    if (!(await toast.isExisting())) return false;
    if (message) {
      const text = await toast.getText();
      return text.includes(message);
    }
    return true;
  }

  /**
   * Wait for loading to complete
   */
  async waitForLoadingComplete(): Promise<void> {
    const loading = await $('[data-testid="loading"]');
    if (await loading.isExisting()) {
      await loading.waitForDisplayed({ timeout: 30000, reverse: true });
    }
  }

  /**
   * Take a screenshot with timestamp
   */
  async takeScreenshot(name: string): Promise<string> {
    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const path = `./e2e/screenshots/${name}-${timestamp}.png`;
    await browser.saveScreenshot(path);
    return path;
  }
}

export default new BasePage();
