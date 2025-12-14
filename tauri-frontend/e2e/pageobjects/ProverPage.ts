/**
 * Prover Page Object
 *
 * @description Page object for the 6-step prover workflow
 */

import { BasePage } from './BasePage';

class ProverPage extends BasePage {
  // =============
  // Selectors
  // =============

  get stepperContainer() {
    return $('[data-testid="workflow-stepper"]');
  }

  get currentStepIndicator() {
    return $('[data-testid="current-step"]');
  }

  get nextButton() {
    return $('[data-testid="next-step-btn"]');
  }

  get prevButton() {
    return $('[data-testid="prev-step-btn"]');
  }

  // Step 1: Import
  get importView() {
    return $('[data-testid="import-view"]');
  }

  get suppliersCsvInput() {
    return $('[data-testid="suppliers-csv-input"]');
  }

  get ubosCsvInput() {
    return $('[data-testid="ubos-csv-input"]');
  }

  get importButton() {
    return $('button*=Importieren');
  }

  get importStatus() {
    return $('[data-testid="import-status"]');
  }

  // Step 2: Commitments
  get commitmentsView() {
    return $('[data-testid="commitments-view"]');
  }

  get createCommitmentsButton() {
    return $('button*=Commitments erstellen');
  }

  get supplierRootHash() {
    return $('[data-testid="supplier-root-hash"]');
  }

  get uboRootHash() {
    return $('[data-testid="ubo-root-hash"]');
  }

  // Step 3: Policy
  get policyView() {
    return $('[data-testid="policy-view"]');
  }

  get policyFileInput() {
    return $('[data-testid="policy-file-input"]');
  }

  get loadPolicyButton() {
    return $('button*=Policy laden');
  }

  get policyName() {
    return $('[data-testid="policy-name"]');
  }

  get policyHash() {
    return $('[data-testid="policy-hash"]');
  }

  // Step 4: Manifest
  get manifestView() {
    return $('[data-testid="manifest-view"]');
  }

  get buildManifestButton() {
    return $('button*=Manifest erstellen');
  }

  get manifestHash() {
    return $('[data-testid="manifest-hash"]');
  }

  // Step 5: Proof
  get proofView() {
    return $('[data-testid="proof-view"]');
  }

  get buildProofButton() {
    return $('button*=Proof generieren');
  }

  get proofProgress() {
    return $('[data-testid="proof-progress"]');
  }

  get proofHash() {
    return $('[data-testid="proof-hash"]');
  }

  // Step 6: Export
  get exportView() {
    return $('[data-testid="export-view"]');
  }

  get exportBundleButton() {
    return $('button*=Bundle exportieren');
  }

  get exportPath() {
    return $('[data-testid="export-path"]');
  }

  get exportSize() {
    return $('[data-testid="export-size"]');
  }

  // =============
  // Actions
  // =============

  /**
   * Navigate to prover workflow
   */
  async navigateToProver(): Promise<void> {
    const proverLink = await $('a*=Prover');
    if (await proverLink.isExisting()) {
      await proverLink.click();
    }
    await this.importView.waitForDisplayed({ timeout: 10000 });
  }

  /**
   * Get current step number
   */
  async getCurrentStep(): Promise<number> {
    const indicator = await this.currentStepIndicator;
    const text = await indicator.getText();
    const match = text.match(/Schritt (\d+)/);
    return match ? parseInt(match[1], 10) : 0;
  }

  /**
   * Go to next step
   */
  async goToNextStep(): Promise<void> {
    const nextBtn = await this.nextButton;
    await nextBtn.waitForClickable({ timeout: 5000 });
    await nextBtn.click();
    await browser.pause(500); // Wait for transition
  }

  /**
   * Go to previous step
   */
  async goToPreviousStep(): Promise<void> {
    const prevBtn = await this.prevButton;
    await prevBtn.waitForClickable({ timeout: 5000 });
    await prevBtn.click();
    await browser.pause(500);
  }

  /**
   * Step 1: Import CSV files
   */
  async importSuppliersCsv(filePath: string): Promise<void> {
    // Note: File input handling in Tauri requires special handling
    // The actual file dialog is mocked in E2E tests
    const importBtn = await this.importButton;
    await importBtn.waitForClickable({ timeout: 5000 });
    await importBtn.click();
    await this.waitForLoadingComplete();
  }

  /**
   * Step 2: Create commitments
   */
  async createCommitments(): Promise<void> {
    await this.commitmentsView.waitForDisplayed({ timeout: 10000 });
    const createBtn = await this.createCommitmentsButton;
    await createBtn.waitForClickable({ timeout: 5000 });
    await createBtn.click();
    await this.waitForLoadingComplete();
  }

  /**
   * Verify commitments were created
   */
  async verifyCommitmentsCreated(): Promise<boolean> {
    const supplierRoot = await this.supplierRootHash;
    const uboRoot = await this.uboRootHash;
    const supplierText = await supplierRoot.getText();
    const uboText = await uboRoot.getText();
    return supplierText.startsWith('0x') && uboText.startsWith('0x');
  }

  /**
   * Step 3: Load policy
   */
  async loadPolicy(): Promise<void> {
    await this.policyView.waitForDisplayed({ timeout: 10000 });
    const loadBtn = await this.loadPolicyButton;
    await loadBtn.waitForClickable({ timeout: 5000 });
    await loadBtn.click();
    await this.waitForLoadingComplete();
  }

  /**
   * Verify policy was loaded
   */
  async verifyPolicyLoaded(): Promise<boolean> {
    const policyName = await this.policyName;
    const text = await policyName.getText();
    return text.length > 0;
  }

  /**
   * Step 4: Build manifest
   */
  async buildManifest(): Promise<void> {
    await this.manifestView.waitForDisplayed({ timeout: 10000 });
    const buildBtn = await this.buildManifestButton;
    await buildBtn.waitForClickable({ timeout: 5000 });
    await buildBtn.click();
    await this.waitForLoadingComplete();
  }

  /**
   * Verify manifest was built
   */
  async verifyManifestBuilt(): Promise<boolean> {
    const hash = await this.manifestHash;
    const text = await hash.getText();
    return text.startsWith('0x') && text.length > 10;
  }

  /**
   * Step 5: Build proof
   */
  async buildProof(): Promise<void> {
    await this.proofView.waitForDisplayed({ timeout: 10000 });
    const buildBtn = await this.buildProofButton;
    await buildBtn.waitForClickable({ timeout: 5000 });
    await buildBtn.click();

    // Wait for proof generation (can take time with real backend)
    await browser.waitUntil(
      async () => {
        const hash = await this.proofHash;
        if (!(await hash.isExisting())) return false;
        const text = await hash.getText();
        return text.startsWith('0x');
      },
      {
        timeout: 120000, // 2 minutes for proof generation
        timeoutMsg: 'Proof generation did not complete',
      }
    );
  }

  /**
   * Get proof progress percentage
   */
  async getProofProgress(): Promise<number> {
    const progress = await this.proofProgress;
    if (!(await progress.isExisting())) return 0;
    const text = await progress.getText();
    const match = text.match(/(\d+)%/);
    return match ? parseInt(match[1], 10) : 0;
  }

  /**
   * Step 6: Export bundle
   */
  async exportBundle(): Promise<void> {
    await this.exportView.waitForDisplayed({ timeout: 10000 });
    const exportBtn = await this.exportBundleButton;
    await exportBtn.waitForClickable({ timeout: 5000 });
    await exportBtn.click();
    await this.waitForLoadingComplete();
  }

  /**
   * Verify bundle was exported
   */
  async verifyBundleExported(): Promise<boolean> {
    const path = await this.exportPath;
    const text = await path.getText();
    return text.endsWith('.zip');
  }

  /**
   * Get exported bundle size
   */
  async getExportedBundleSize(): Promise<string> {
    const size = await this.exportSize;
    return await size.getText();
  }

  /**
   * Complete full 6-step workflow
   */
  async completeFullWorkflow(): Promise<void> {
    // Step 1: Import
    await this.importSuppliersCsv('fixtures/suppliers.csv');
    await this.goToNextStep();

    // Step 2: Commitments
    await this.createCommitments();
    await this.goToNextStep();

    // Step 3: Policy
    await this.loadPolicy();
    await this.goToNextStep();

    // Step 4: Manifest
    await this.buildManifest();
    await this.goToNextStep();

    // Step 5: Proof
    await this.buildProof();
    await this.goToNextStep();

    // Step 6: Export
    await this.exportBundle();
  }
}

export default new ProverPage();
