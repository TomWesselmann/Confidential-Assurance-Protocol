/**
 * CAP Workflow Store Tests
 *
 * @description Comprehensive unit tests for Zustand workflow state management
 */

import { describe, it, expect, beforeEach } from 'vitest';
import { useWorkflowStore, STEP_ORDER, selectCurrentStepIndex, selectIsFirstStep, selectIsLastStep, selectCanProceed, selectStepLabel } from '../../store/workflowStore';
import type { WorkflowStep, WorkflowState, ProjectStatus } from '../../store/workflowStore';

describe('workflowStore', () => {
  beforeEach(() => {
    // Reset store before each test
    useWorkflowStore.getState().reset();
  });

  describe('initial state', () => {
    it('should have null project path initially', () => {
      const state = useWorkflowStore.getState();
      expect(state.projectPath).toBeNull();
    });

    it('should have null project name initially', () => {
      const state = useWorkflowStore.getState();
      expect(state.projectName).toBeNull();
    });

    it('should start at import step', () => {
      const state = useWorkflowStore.getState();
      expect(state.currentStep).toBe('import');
    });

    it('should have all steps pending initially', () => {
      const state = useWorkflowStore.getState();
      STEP_ORDER.forEach((step) => {
        expect(state.steps[step].status).toBe('pending');
      });
    });

    it('should have null import results initially', () => {
      const state = useWorkflowStore.getState();
      expect(state.importResults.suppliers).toBeNull();
      expect(state.importResults.ubos).toBeNull();
    });

    it('should have null commitments result initially', () => {
      const state = useWorkflowStore.getState();
      expect(state.commitmentsResult).toBeNull();
    });

    it('should have null policy info initially', () => {
      const state = useWorkflowStore.getState();
      expect(state.policyInfo).toBeNull();
    });

    it('should have null manifest result initially', () => {
      const state = useWorkflowStore.getState();
      expect(state.manifestResult).toBeNull();
    });

    it('should have null proof result initially', () => {
      const state = useWorkflowStore.getState();
      expect(state.proofResult).toBeNull();
    });

    it('should have null export result initially', () => {
      const state = useWorkflowStore.getState();
      expect(state.exportResult).toBeNull();
    });
  });

  describe('setProject', () => {
    it('should set project path and name', () => {
      useWorkflowStore.getState().setProject('/path/to/project', 'Test Project');

      const state = useWorkflowStore.getState();
      expect(state.projectPath).toBe('/path/to/project');
      expect(state.projectName).toBe('Test Project');
    });

    it('should reset state when switching to different project', () => {
      // Set first project and make changes
      useWorkflowStore.getState().setProject('/project1', 'Project 1');
      useWorkflowStore.getState().setCurrentStep('commitments');

      // Switch to different project
      useWorkflowStore.getState().setProject('/project2', 'Project 2');

      const state = useWorkflowStore.getState();
      expect(state.projectPath).toBe('/project2');
      expect(state.currentStep).toBe('import'); // Should reset
    });

    it('should not reset state when setting same project', () => {
      // Set project and make changes
      useWorkflowStore.getState().setProject('/project1', 'Project 1');
      useWorkflowStore.getState().setCurrentStep('commitments');

      // Set same project again
      useWorkflowStore.getState().setProject('/project1', 'Project 1');

      const state = useWorkflowStore.getState();
      expect(state.currentStep).toBe('commitments'); // Should NOT reset
    });
  });

  describe('initializeFromStatus', () => {
    it('should initialize from project status with completed steps', () => {
      const status: ProjectStatus = {
        hasSuppliersCSv: true,
        hasUbosCsv: true,
        hasPolicy: true,
        hasCommitments: true,
        hasManifest: false,
        hasProof: false,
        currentStep: 'manifest',
      };

      useWorkflowStore.getState().initializeFromStatus('/project', 'Test', status);

      const state = useWorkflowStore.getState();
      expect(state.steps.import.status).toBe('completed');
      expect(state.steps.commitments.status).toBe('completed');
      expect(state.steps.policy.status).toBe('completed');
      expect(state.steps.manifest.status).toBe('pending');
      expect(state.currentStep).toBe('manifest');
    });

    it('should set import in_progress when only suppliers CSV exists', () => {
      const status: ProjectStatus = {
        hasSuppliersCSv: true,
        hasUbosCsv: false,
        hasPolicy: false,
        hasCommitments: false,
        hasManifest: false,
        hasProof: false,
        currentStep: 'import',
      };

      useWorkflowStore.getState().initializeFromStatus('/project', 'Test', status);

      const state = useWorkflowStore.getState();
      expect(state.steps.import.status).toBe('in_progress');
    });

    it('should not reset when same project', () => {
      useWorkflowStore.getState().setProject('/project', 'Test');
      useWorkflowStore.getState().setCurrentStep('proof');

      const status: ProjectStatus = {
        hasSuppliersCSv: true,
        hasUbosCsv: true,
        hasPolicy: true,
        hasCommitments: true,
        hasManifest: false,
        hasProof: false,
        currentStep: 'import',
      };

      useWorkflowStore.getState().initializeFromStatus('/project', 'Test', status);

      const state = useWorkflowStore.getState();
      expect(state.currentStep).toBe('proof'); // Should NOT change
    });
  });

  describe('setCurrentStep', () => {
    it('should change current step', () => {
      useWorkflowStore.getState().setCurrentStep('policy');

      expect(useWorkflowStore.getState().currentStep).toBe('policy');
    });
  });

  describe('setStepStatus', () => {
    it('should update step status', () => {
      useWorkflowStore.getState().setStepStatus('import', 'completed');

      expect(useWorkflowStore.getState().steps.import.status).toBe('completed');
    });

    it('should update step status with error', () => {
      useWorkflowStore.getState().setStepStatus('import', 'error', 'Test error');

      const state = useWorkflowStore.getState();
      expect(state.steps.import.status).toBe('error');
      expect(state.steps.import.error).toBe('Test error');
    });
  });

  describe('setImportResult', () => {
    it('should set suppliers import result', () => {
      const result = {
        csv_type: 'suppliers',
        record_count: 10,
        hash: '0x1234',
        destination: '/path/to/file',
      };

      useWorkflowStore.getState().setImportResult('suppliers', result);

      const state = useWorkflowStore.getState();
      expect(state.importResults.suppliers).toEqual(result);
      expect(state.steps.import.status).toBe('in_progress');
    });

    it('should complete import step when both CSVs imported', () => {
      const suppliersResult = {
        csv_type: 'suppliers',
        record_count: 10,
        hash: '0x1234',
        destination: '/path/suppliers',
      };
      const ubosResult = {
        csv_type: 'ubos',
        record_count: 5,
        hash: '0x5678',
        destination: '/path/ubos',
      };

      useWorkflowStore.getState().setImportResult('suppliers', suppliersResult);
      useWorkflowStore.getState().setImportResult('ubos', ubosResult);

      const state = useWorkflowStore.getState();
      expect(state.steps.import.status).toBe('completed');
    });
  });

  describe('setCommitmentsResult', () => {
    it('should set commitments result and complete step', () => {
      const result = {
        supplier_root: '0xabc',
        ubo_root: '0xdef',
        company_root: '0x123',
        path: '/path/to/commitments',
      };

      useWorkflowStore.getState().setCommitmentsResult(result);

      const state = useWorkflowStore.getState();
      expect(state.commitmentsResult).toEqual(result);
      expect(state.steps.commitments.status).toBe('completed');
    });
  });

  describe('setPolicyInfo', () => {
    it('should set policy info and complete step', () => {
      const info = {
        name: 'LkSG Policy',
        version: 'lksg.v1',
        hash: '0xpolicy',
        rules_count: 15,
        path: '/path/to/policy',
      };

      useWorkflowStore.getState().setPolicyInfo(info);

      const state = useWorkflowStore.getState();
      expect(state.policyInfo).toEqual(info);
      expect(state.steps.policy.status).toBe('completed');
    });
  });

  describe('setManifestResult', () => {
    it('should set manifest result and complete step', () => {
      const result = {
        manifest_hash: '0xmanifest',
        path: '/path/to/manifest',
        supplier_root: '0xsupplier',
        ubo_root: '0xubo',
        policy_hash: '0xpolicy',
      };

      useWorkflowStore.getState().setManifestResult(result);

      const state = useWorkflowStore.getState();
      expect(state.manifestResult).toEqual(result);
      expect(state.steps.manifest.status).toBe('completed');
    });
  });

  describe('setProofResult', () => {
    it('should set proof result, clear progress, and complete step', () => {
      // First set progress
      useWorkflowStore.getState().setProofProgress({ percent: 50, message: 'Working...' });

      const result = {
        proof_hash: '0xproof',
        path: '/path/to/proof',
        backend: 'mock',
      };

      useWorkflowStore.getState().setProofResult(result);

      const state = useWorkflowStore.getState();
      expect(state.proofResult).toEqual(result);
      expect(state.proofProgress).toBeNull(); // Should be cleared
      expect(state.steps.proof.status).toBe('completed');
    });
  });

  describe('setProofProgress', () => {
    it('should set proof progress', () => {
      useWorkflowStore.getState().setProofProgress({ percent: 75, message: 'Processing...' });

      const state = useWorkflowStore.getState();
      expect(state.proofProgress?.percent).toBe(75);
      expect(state.proofProgress?.message).toBe('Processing...');
    });

    it('should clear proof progress when null', () => {
      useWorkflowStore.getState().setProofProgress({ percent: 50, message: 'Test' });
      useWorkflowStore.getState().setProofProgress(null);

      expect(useWorkflowStore.getState().proofProgress).toBeNull();
    });
  });

  describe('setExportResult', () => {
    it('should set export result and complete step', () => {
      const result = {
        bundle_path: '/path/to/bundle.zip',
        size_bytes: 12345,
        hash: '0xbundle',
        files: ['manifest.json', 'proof.bin'],
      };

      useWorkflowStore.getState().setExportResult(result);

      const state = useWorkflowStore.getState();
      expect(state.exportResult).toEqual(result);
      expect(state.steps.export.status).toBe('completed');
    });
  });

  describe('canGoToStep', () => {
    it('should allow going back to previous steps', () => {
      useWorkflowStore.getState().setCurrentStep('manifest');

      expect(useWorkflowStore.getState().canGoToStep('import')).toBe(true);
      expect(useWorkflowStore.getState().canGoToStep('commitments')).toBe(true);
    });

    it('should allow going to current step', () => {
      useWorkflowStore.getState().setCurrentStep('policy');

      expect(useWorkflowStore.getState().canGoToStep('policy')).toBe(true);
    });

    it('should allow going to next step if current is completed', () => {
      useWorkflowStore.getState().setStepStatus('import', 'completed');

      expect(useWorkflowStore.getState().canGoToStep('commitments')).toBe(true);
    });

    it('should not allow skipping steps without completion', () => {
      expect(useWorkflowStore.getState().canGoToStep('manifest')).toBe(false);
    });

    it('should allow going to any step if all previous are completed', () => {
      useWorkflowStore.getState().setStepStatus('import', 'completed');
      useWorkflowStore.getState().setStepStatus('commitments', 'completed');
      useWorkflowStore.getState().setStepStatus('policy', 'completed');

      expect(useWorkflowStore.getState().canGoToStep('manifest')).toBe(true);
    });
  });

  describe('goToNextStep', () => {
    it('should go to next step when current is completed', () => {
      useWorkflowStore.getState().setStepStatus('import', 'completed');
      useWorkflowStore.getState().goToNextStep();

      expect(useWorkflowStore.getState().currentStep).toBe('commitments');
    });

    it('should not go to next step if current is not completed', () => {
      useWorkflowStore.getState().goToNextStep();

      expect(useWorkflowStore.getState().currentStep).toBe('import');
    });

    it('should not go beyond last step', () => {
      // Complete all steps and go to export
      STEP_ORDER.forEach((step) => {
        useWorkflowStore.getState().setStepStatus(step, 'completed');
      });
      useWorkflowStore.getState().setCurrentStep('export');

      useWorkflowStore.getState().goToNextStep();

      expect(useWorkflowStore.getState().currentStep).toBe('export');
    });
  });

  describe('goToPreviousStep', () => {
    it('should go to previous step', () => {
      useWorkflowStore.getState().setCurrentStep('policy');
      useWorkflowStore.getState().goToPreviousStep();

      expect(useWorkflowStore.getState().currentStep).toBe('commitments');
    });

    it('should not go before first step', () => {
      useWorkflowStore.getState().goToPreviousStep();

      expect(useWorkflowStore.getState().currentStep).toBe('import');
    });
  });

  describe('reset', () => {
    it('should reset all state to initial values', () => {
      // Set various state
      useWorkflowStore.getState().setProject('/project', 'Test');
      useWorkflowStore.getState().setCurrentStep('manifest');
      useWorkflowStore.getState().setStepStatus('import', 'completed');

      // Reset
      useWorkflowStore.getState().reset();

      const state = useWorkflowStore.getState();
      expect(state.projectPath).toBeNull();
      expect(state.projectName).toBeNull();
      expect(state.currentStep).toBe('import');
      expect(state.steps.import.status).toBe('pending');
    });
  });

  describe('selectors', () => {
    it('selectCurrentStepIndex should return correct index', () => {
      useWorkflowStore.getState().setCurrentStep('policy');
      const state = useWorkflowStore.getState();

      expect(selectCurrentStepIndex(state)).toBe(2);
    });

    it('selectIsFirstStep should return true for import', () => {
      const state = useWorkflowStore.getState();

      expect(selectIsFirstStep(state)).toBe(true);
    });

    it('selectIsFirstStep should return false for other steps', () => {
      useWorkflowStore.getState().setCurrentStep('commitments');
      const state = useWorkflowStore.getState();

      expect(selectIsFirstStep(state)).toBe(false);
    });

    it('selectIsLastStep should return true for export', () => {
      useWorkflowStore.getState().setCurrentStep('export');
      const state = useWorkflowStore.getState();

      expect(selectIsLastStep(state)).toBe(true);
    });

    it('selectIsLastStep should return false for other steps', () => {
      const state = useWorkflowStore.getState();

      expect(selectIsLastStep(state)).toBe(false);
    });

    it('selectCanProceed should return true when step completed', () => {
      useWorkflowStore.getState().setStepStatus('import', 'completed');
      const state = useWorkflowStore.getState();

      expect(selectCanProceed(state)).toBe(true);
    });

    it('selectCanProceed should return false when step not completed', () => {
      const state = useWorkflowStore.getState();

      expect(selectCanProceed(state)).toBe(false);
    });

    it('selectStepLabel should return correct labels', () => {
      expect(selectStepLabel('import')).toBe('Import');
      expect(selectStepLabel('commitments')).toBe('Commitments');
      expect(selectStepLabel('policy')).toBe('Policy');
      expect(selectStepLabel('manifest')).toBe('Manifest');
      expect(selectStepLabel('proof')).toBe('Proof');
      expect(selectStepLabel('export')).toBe('Export');
    });
  });

  describe('STEP_ORDER', () => {
    it('should have correct step order', () => {
      expect(STEP_ORDER).toEqual([
        'import',
        'commitments',
        'policy',
        'manifest',
        'proof',
        'export',
      ]);
    });

    it('should have 6 steps', () => {
      expect(STEP_ORDER.length).toBe(6);
    });
  });
});
