/**
 * Verifier Page Object
 *
 * @description Page object for bundle verification workflow
 */

import { BasePage } from './BasePage';

class VerifierPage extends BasePage {
  // =============
  // Selectors
  // =============

  get verifierView() {
    return $('[data-testid="verifier-view"]');
  }

  get bundleDropzone() {
    return $('[data-testid="bundle-dropzone"]');
  }

  get selectBundleButton() {
    return $('button*=Bundle auswählen');
  }

  get verifyButton() {
    return $('button*=Verifizieren');
  }

  get verificationResult() {
    return $('[data-testid="verification-result"]');
  }

  get verificationStatus() {
    return $('[data-testid="verification-status"]');
  }

  get verificationStatusIcon() {
    return $('[data-testid="verification-status-icon"]');
  }

  get manifestHashDisplay() {
    return $('[data-testid="manifest-hash-display"]');
  }

  get proofHashDisplay() {
    return $('[data-testid="proof-hash-display"]');
  }

  get signatureStatus() {
    return $('[data-testid="signature-status"]');
  }

  get constraintResults() {
    return $('[data-testid="constraint-results"]');
  }

  get constraintItems() {
    return $$('[data-testid="constraint-item"]');
  }

  get checksPassedCount() {
    return $('[data-testid="checks-passed"]');
  }

  get checksTotalCount() {
    return $('[data-testid="checks-total"]');
  }

  get errorMessage() {
    return $('[data-testid="error-message"]');
  }

  get bundleInfo() {
    return $('[data-testid="bundle-info"]');
  }

  get bundleId() {
    return $('[data-testid="bundle-id"]');
  }

  get bundleCreatedAt() {
    return $('[data-testid="bundle-created-at"]');
  }

  get resetButton() {
    return $('button*=Zurücksetzen');
  }

  // =============
  // Actions
  // =============

  /**
   * Navigate to verifier view
   */
  async navigateToVerifier(): Promise<void> {
    const verifierLink = await $('a*=Verifier');
    if (await verifierLink.isExisting()) {
      await verifierLink.click();
    }
    await this.verifierView.waitForDisplayed({ timeout: 10000 });
  }

  /**
   * Check if bundle dropzone is visible
   */
  async isDropzoneVisible(): Promise<boolean> {
    const dropzone = await this.bundleDropzone;
    return await dropzone.isDisplayed();
  }

  /**
   * Select a bundle file for verification
   * Note: Actual file dialog is handled by Tauri
   */
  async selectBundle(): Promise<void> {
    const selectBtn = await this.selectBundleButton;
    await selectBtn.waitForClickable({ timeout: 5000 });
    await selectBtn.click();
    // File dialog will be handled by Tauri
  }

  /**
   * Start verification process
   */
  async startVerification(): Promise<void> {
    const verifyBtn = await this.verifyButton;
    await verifyBtn.waitForClickable({ timeout: 5000 });
    await verifyBtn.click();
    await this.waitForLoadingComplete();
  }

  /**
   * Wait for verification result
   */
  async waitForVerificationResult(): Promise<void> {
    await browser.waitUntil(
      async () => {
        const result = await this.verificationResult;
        return await result.isExisting();
      },
      {
        timeout: 60000,
        timeoutMsg: 'Verification result did not appear',
      }
    );
  }

  /**
   * Get verification status (ok/fail)
   */
  async getVerificationStatus(): Promise<string> {
    const status = await this.verificationStatus;
    const text = await status.getText();
    return text.toLowerCase();
  }

  /**
   * Check if verification passed
   */
  async isVerificationPassed(): Promise<boolean> {
    const status = await this.getVerificationStatus();
    return status.includes('gültig') || status.includes('valid') || status.includes('ok');
  }

  /**
   * Check if verification failed
   */
  async isVerificationFailed(): Promise<boolean> {
    const status = await this.getVerificationStatus();
    return status.includes('ungültig') || status.includes('invalid') || status.includes('fail');
  }

  /**
   * Get manifest hash from result
   */
  async getManifestHash(): Promise<string> {
    const hash = await this.manifestHashDisplay;
    return await hash.getText();
  }

  /**
   * Get proof hash from result
   */
  async getProofHash(): Promise<string> {
    const hash = await this.proofHashDisplay;
    return await hash.getText();
  }

  /**
   * Check if signature is valid
   */
  async isSignatureValid(): Promise<boolean> {
    const sigStatus = await this.signatureStatus;
    const text = await sigStatus.getText();
    return text.toLowerCase().includes('gültig') || text.toLowerCase().includes('valid');
  }

  /**
   * Get number of checks passed
   */
  async getChecksPassedCount(): Promise<number> {
    const passed = await this.checksPassedCount;
    const text = await passed.getText();
    return parseInt(text, 10) || 0;
  }

  /**
   * Get total number of checks
   */
  async getChecksTotalCount(): Promise<number> {
    const total = await this.checksTotalCount;
    const text = await total.getText();
    return parseInt(text, 10) || 0;
  }

  /**
   * Get all constraint results
   */
  async getConstraintResults(): Promise<Array<{ name: string; status: string }>> {
    const items = await this.constraintItems;
    const results: Array<{ name: string; status: string }> = [];

    for (const item of items) {
      const name = await item.$('[data-testid="constraint-name"]').getText();
      const status = await item.$('[data-testid="constraint-status"]').getText();
      results.push({ name, status });
    }

    return results;
  }

  /**
   * Get error message if verification failed
   */
  async getErrorMessage(): Promise<string | null> {
    const error = await this.errorMessage;
    if (await error.isExisting()) {
      return await error.getText();
    }
    return null;
  }

  /**
   * Get bundle ID from info
   */
  async getBundleId(): Promise<string> {
    const id = await this.bundleId;
    return await id.getText();
  }

  /**
   * Get bundle creation date
   */
  async getBundleCreatedAt(): Promise<string> {
    const date = await this.bundleCreatedAt;
    return await date.getText();
  }

  /**
   * Reset verifier to initial state
   */
  async reset(): Promise<void> {
    const resetBtn = await this.resetButton;
    if (await resetBtn.isExisting()) {
      await resetBtn.click();
      await browser.pause(500);
    }
  }

  /**
   * Complete full verification workflow
   * @returns Object with verification results
   */
  async completeVerification(): Promise<{
    passed: boolean;
    manifestHash: string;
    proofHash: string;
    checksTotal: number;
    checksPassed: number;
  }> {
    await this.selectBundle();
    await browser.pause(1000); // Wait for file dialog
    await this.startVerification();
    await this.waitForVerificationResult();

    return {
      passed: await this.isVerificationPassed(),
      manifestHash: await this.getManifestHash(),
      proofHash: await this.getProofHash(),
      checksTotal: await this.getChecksTotalCount(),
      checksPassed: await this.getChecksPassedCount(),
    };
  }
}

export default new VerifierPage();
