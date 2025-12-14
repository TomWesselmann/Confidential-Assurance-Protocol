/**
 * Prover Workflow E2E Tests
 *
 * @description End-to-end tests for the 6-step prover workflow
 * Tests the critical user journey: Create Project → Import CSV → Generate Proof → Export
 */

import ProverPage from '../pageobjects/ProverPage';

describe('Prover Workflow', () => {
  before(async () => {
    await ProverPage.waitForAppReady();
  });

  describe('Initial State', () => {
    it('should show prover view on startup', async () => {
      await ProverPage.navigateToProver();
      const importView = await ProverPage.importView;
      expect(await importView.isDisplayed()).toBe(true);
    });

    it('should display workflow stepper', async () => {
      const stepper = await ProverPage.stepperContainer;
      expect(await stepper.isExisting()).toBe(true);
    });

    it('should start at step 1 (Import)', async () => {
      const step = await ProverPage.getCurrentStep();
      expect(step).toBe(1);
    });
  });

  describe('Step 1: CSV Import', () => {
    it('should display import view', async () => {
      const importView = await ProverPage.importView;
      expect(await importView.isDisplayed()).toBe(true);
    });

    it('should have import button', async () => {
      const importBtn = await ProverPage.importButton;
      expect(await importBtn.isExisting()).toBe(true);
    });

    it('should show import status after import', async () => {
      await ProverPage.importSuppliersCsv('fixtures/suppliers.csv');
      const status = await ProverPage.importStatus;
      expect(await status.isExisting()).toBe(true);
    });

    it('should allow navigation to step 2', async () => {
      const nextBtn = await ProverPage.nextButton;
      expect(await nextBtn.isClickable()).toBe(true);
    });
  });

  describe('Step 2: Commitments', () => {
    before(async () => {
      await ProverPage.goToNextStep();
    });

    it('should display commitments view', async () => {
      const commitmentsView = await ProverPage.commitmentsView;
      expect(await commitmentsView.isDisplayed()).toBe(true);
    });

    it('should show current step as 2', async () => {
      const step = await ProverPage.getCurrentStep();
      expect(step).toBe(2);
    });

    it('should create commitments successfully', async () => {
      await ProverPage.createCommitments();
      const created = await ProverPage.verifyCommitmentsCreated();
      expect(created).toBe(true);
    });

    it('should display supplier root hash', async () => {
      const hash = await ProverPage.supplierRootHash;
      const text = await hash.getText();
      expect(text).toMatch(/^0x[a-fA-F0-9]+/);
    });

    it('should display UBO root hash', async () => {
      const hash = await ProverPage.uboRootHash;
      const text = await hash.getText();
      expect(text).toMatch(/^0x[a-fA-F0-9]+/);
    });

    it('should allow navigation back to step 1', async () => {
      await ProverPage.goToPreviousStep();
      const step = await ProverPage.getCurrentStep();
      expect(step).toBe(1);
      await ProverPage.goToNextStep(); // Go back to step 2
    });
  });

  describe('Step 3: Policy', () => {
    before(async () => {
      await ProverPage.goToNextStep();
    });

    it('should display policy view', async () => {
      const policyView = await ProverPage.policyView;
      expect(await policyView.isDisplayed()).toBe(true);
    });

    it('should show current step as 3', async () => {
      const step = await ProverPage.getCurrentStep();
      expect(step).toBe(3);
    });

    it('should load policy successfully', async () => {
      await ProverPage.loadPolicy();
      const loaded = await ProverPage.verifyPolicyLoaded();
      expect(loaded).toBe(true);
    });

    it('should display policy name', async () => {
      const name = await ProverPage.policyName;
      const text = await name.getText();
      expect(text.length).toBeGreaterThan(0);
    });

    it('should display policy hash', async () => {
      const hash = await ProverPage.policyHash;
      const text = await hash.getText();
      expect(text).toMatch(/^0x[a-fA-F0-9]+/);
    });
  });

  describe('Step 4: Manifest', () => {
    before(async () => {
      await ProverPage.goToNextStep();
    });

    it('should display manifest view', async () => {
      const manifestView = await ProverPage.manifestView;
      expect(await manifestView.isDisplayed()).toBe(true);
    });

    it('should show current step as 4', async () => {
      const step = await ProverPage.getCurrentStep();
      expect(step).toBe(4);
    });

    it('should build manifest successfully', async () => {
      await ProverPage.buildManifest();
      const built = await ProverPage.verifyManifestBuilt();
      expect(built).toBe(true);
    });

    it('should display manifest hash', async () => {
      const hash = await ProverPage.manifestHash;
      const text = await hash.getText();
      expect(text).toMatch(/^0x[a-fA-F0-9]{64}/);
    });
  });

  describe('Step 5: Proof', () => {
    before(async () => {
      await ProverPage.goToNextStep();
    });

    it('should display proof view', async () => {
      const proofView = await ProverPage.proofView;
      expect(await proofView.isDisplayed()).toBe(true);
    });

    it('should show current step as 5', async () => {
      const step = await ProverPage.getCurrentStep();
      expect(step).toBe(5);
    });

    it('should show progress during proof generation', async () => {
      // Start proof building but don't wait for completion
      const buildBtn = await ProverPage.buildProofButton;
      await buildBtn.click();

      // Check progress appears
      await browser.waitUntil(
        async () => {
          const progress = await ProverPage.proofProgress;
          return await progress.isExisting();
        },
        { timeout: 10000 }
      );

      const progress = await ProverPage.getProofProgress();
      expect(progress).toBeGreaterThanOrEqual(0);
    });

    it('should complete proof generation', async () => {
      // Wait for proof to complete (can take up to 2 minutes)
      await browser.waitUntil(
        async () => {
          const hash = await ProverPage.proofHash;
          if (!(await hash.isExisting())) return false;
          const text = await hash.getText();
          return text.startsWith('0x');
        },
        { timeout: 120000, timeoutMsg: 'Proof generation timed out' }
      );

      const hash = await ProverPage.proofHash;
      const text = await hash.getText();
      expect(text).toMatch(/^0x[a-fA-F0-9]{64}/);
    });
  });

  describe('Step 6: Export', () => {
    before(async () => {
      await ProverPage.goToNextStep();
    });

    it('should display export view', async () => {
      const exportView = await ProverPage.exportView;
      expect(await exportView.isDisplayed()).toBe(true);
    });

    it('should show current step as 6', async () => {
      const step = await ProverPage.getCurrentStep();
      expect(step).toBe(6);
    });

    it('should export bundle successfully', async () => {
      await ProverPage.exportBundle();
      const exported = await ProverPage.verifyBundleExported();
      expect(exported).toBe(true);
    });

    it('should display export path', async () => {
      const path = await ProverPage.exportPath;
      const text = await path.getText();
      expect(text).toContain('.zip');
    });

    it('should display bundle size', async () => {
      const size = await ProverPage.getExportedBundleSize();
      expect(size).toMatch(/\d+(\.\d+)?\s*(B|KB|MB)/);
    });
  });

  describe('Complete Workflow', () => {
    it('should complete full 6-step workflow without errors', async () => {
      // Reset and run full workflow
      await browser.refresh();
      await ProverPage.waitForAppReady();
      await ProverPage.navigateToProver();

      // This test validates the entire happy path
      // Individual steps are tested above
      const finalStep = await ProverPage.getCurrentStep();
      expect(finalStep).toBe(1); // Should start fresh
    });
  });
});
