/**
 * CAP Proofer Workflow Store
 *
 * @description Zustand state management for the 6-step Proofer workflow
 * @architecture Imperative Shell (manages UI state)
 */

import { create } from 'zustand';
import type {
  ImportResult,
  CommitmentsResult,
  PolicyInfo,
  ManifestResult,
  ProofResult,
  ExportResult,
  ProofProgress,
} from '../lib/tauri';

// ============================================================================
// Types
// ============================================================================

export type WorkflowStep =
  | 'import'
  | 'commitments'
  | 'policy'
  | 'manifest'
  | 'proof'
  | 'export';

export type StepStatus = 'pending' | 'in_progress' | 'completed' | 'error';

export interface StepState {
  status: StepStatus;
  error?: string;
}

export interface ProjectStatus {
  hasSuppliersCSv: boolean;
  hasUbosCsv: boolean;
  hasPolicy: boolean;
  hasCommitments: boolean;
  hasManifest: boolean;
  hasProof: boolean;
  currentStep: string;
}

export interface WorkflowState {
  // Project
  projectPath: string | null;
  projectName: string | null;

  // Current workflow position
  currentStep: WorkflowStep;

  // Step states
  steps: Record<WorkflowStep, StepState>;

  // Step results
  importResults: {
    suppliers: ImportResult | null;
    ubos: ImportResult | null;
  };
  commitmentsResult: CommitmentsResult | null;
  policyInfo: PolicyInfo | null;
  manifestResult: ManifestResult | null;
  proofResult: ProofResult | null;
  proofProgress: ProofProgress | null;
  exportResult: ExportResult | null;

  // Actions
  setProject: (path: string, name: string) => void;
  initializeFromStatus: (path: string, name: string, status: ProjectStatus) => void;
  setCurrentStep: (step: WorkflowStep) => void;
  setStepStatus: (step: WorkflowStep, status: StepStatus, error?: string) => void;

  // Result setters
  setImportResult: (type: 'suppliers' | 'ubos', result: ImportResult) => void;
  setCommitmentsResult: (result: CommitmentsResult) => void;
  setPolicyInfo: (info: PolicyInfo) => void;
  setManifestResult: (result: ManifestResult) => void;
  setProofResult: (result: ProofResult) => void;
  setProofProgress: (progress: ProofProgress | null) => void;
  setExportResult: (result: ExportResult) => void;

  // Navigation
  canGoToStep: (step: WorkflowStep) => boolean;
  goToNextStep: () => void;
  goToPreviousStep: () => void;

  // Reset
  reset: () => void;
}

// ============================================================================
// Initial State
// ============================================================================

const initialSteps: Record<WorkflowStep, StepState> = {
  import: { status: 'pending' },
  commitments: { status: 'pending' },
  policy: { status: 'pending' },
  manifest: { status: 'pending' },
  proof: { status: 'pending' },
  export: { status: 'pending' },
};

const initialState = {
  projectPath: null,
  projectName: null,
  currentStep: 'import' as WorkflowStep,
  steps: { ...initialSteps },
  importResults: {
    suppliers: null,
    ubos: null,
  },
  commitmentsResult: null,
  policyInfo: null,
  manifestResult: null,
  proofResult: null,
  proofProgress: null,
  exportResult: null,
};

// ============================================================================
// Step Order
// ============================================================================

const STEP_ORDER: WorkflowStep[] = [
  'import',
  'commitments',
  'policy',
  'manifest',
  'proof',
  'export',
];

// ============================================================================
// Store
// ============================================================================

export const useWorkflowStore = create<WorkflowState>((set, get) => ({
  ...initialState,

  setProject: (path, name) => {
    const state = get();
    // Only reset if switching to a different project
    if (state.projectPath === path) {
      return; // Same project, keep state
    }
    set({
      // Reset workflow state when project changes
      ...initialState,
      // Then set the new project values
      projectPath: path,
      projectName: name,
    });
  },

  initializeFromStatus: (path, name, status) => {
    const state = get();
    // Only reset if switching to a different project
    if (state.projectPath === path) {
      return; // Same project, keep state
    }

    // Determine step states from backend status
    const newSteps = { ...initialSteps };

    // Import step
    if (status.hasSuppliersCSv && status.hasUbosCsv) {
      newSteps.import = { status: 'completed' };
    } else if (status.hasSuppliersCSv || status.hasUbosCsv) {
      newSteps.import = { status: 'in_progress' };
    }

    // Commitments step
    if (status.hasCommitments) {
      newSteps.commitments = { status: 'completed' };
    }

    // Policy step
    if (status.hasPolicy) {
      newSteps.policy = { status: 'completed' };
    }

    // Manifest step
    if (status.hasManifest) {
      newSteps.manifest = { status: 'completed' };
    }

    // Proof step
    if (status.hasProof) {
      newSteps.proof = { status: 'completed' };
    }

    // Determine current step from backend
    const currentStep = (status.currentStep || 'import') as WorkflowStep;

    set({
      projectPath: path,
      projectName: name,
      currentStep,
      steps: newSteps,
      // Reset results - they will be loaded on demand
      importResults: { suppliers: null, ubos: null },
      commitmentsResult: null,
      policyInfo: null,
      manifestResult: null,
      proofResult: null,
      proofProgress: null,
      exportResult: null,
    });
  },

  setCurrentStep: (step) => set({ currentStep: step }),

  setStepStatus: (step, status, error) =>
    set((state) => ({
      steps: {
        ...state.steps,
        [step]: { status, error },
      },
    })),

  setImportResult: (type, result) =>
    set((state) => {
      const newImportResults = {
        ...state.importResults,
        [type]: result,
      };

      // Check if import step is complete (both CSVs imported)
      const importComplete =
        newImportResults.suppliers !== null && newImportResults.ubos !== null;

      return {
        importResults: newImportResults,
        steps: {
          ...state.steps,
          import: {
            status: importComplete ? 'completed' : 'in_progress',
          },
        },
      };
    }),

  setCommitmentsResult: (result) =>
    set((state) => ({
      commitmentsResult: result,
      steps: {
        ...state.steps,
        commitments: { status: 'completed' },
      },
    })),

  setPolicyInfo: (info) =>
    set((state) => ({
      policyInfo: info,
      steps: {
        ...state.steps,
        policy: { status: 'completed' },
      },
    })),

  setManifestResult: (result) =>
    set((state) => ({
      manifestResult: result,
      steps: {
        ...state.steps,
        manifest: { status: 'completed' },
      },
    })),

  setProofResult: (result) =>
    set((state) => ({
      proofResult: result,
      proofProgress: null,
      steps: {
        ...state.steps,
        proof: { status: 'completed' },
      },
    })),

  setProofProgress: (progress) => set({ proofProgress: progress }),

  setExportResult: (result) =>
    set((state) => ({
      exportResult: result,
      steps: {
        ...state.steps,
        export: { status: 'completed' },
      },
    })),

  canGoToStep: (step) => {
    const state = get();
    const stepIndex = STEP_ORDER.indexOf(step);
    const currentIndex = STEP_ORDER.indexOf(state.currentStep);

    // Can always go back
    if (stepIndex < currentIndex) return true;

    // Can go to current step
    if (stepIndex === currentIndex) return true;

    // Can only go forward if current step is completed
    if (stepIndex === currentIndex + 1) {
      return state.steps[state.currentStep].status === 'completed';
    }

    // For steps further ahead, all previous steps must be completed
    for (let i = 0; i < stepIndex; i++) {
      if (state.steps[STEP_ORDER[i]].status !== 'completed') {
        return false;
      }
    }

    return true;
  },

  goToNextStep: () => {
    const state = get();
    const currentIndex = STEP_ORDER.indexOf(state.currentStep);

    if (currentIndex < STEP_ORDER.length - 1) {
      const nextStep = STEP_ORDER[currentIndex + 1];
      if (state.canGoToStep(nextStep)) {
        set({ currentStep: nextStep });
      }
    }
  },

  goToPreviousStep: () => {
    const state = get();
    const currentIndex = STEP_ORDER.indexOf(state.currentStep);

    if (currentIndex > 0) {
      set({ currentStep: STEP_ORDER[currentIndex - 1] });
    }
  },

  reset: () =>
    set({
      ...initialState,
      steps: { ...initialSteps },
      importResults: { suppliers: null, ubos: null },
    }),
}));

// ============================================================================
// Selectors
// ============================================================================

export const selectCurrentStepIndex = (state: WorkflowState): number =>
  STEP_ORDER.indexOf(state.currentStep);

export const selectIsFirstStep = (state: WorkflowState): boolean =>
  state.currentStep === STEP_ORDER[0];

export const selectIsLastStep = (state: WorkflowState): boolean =>
  state.currentStep === STEP_ORDER[STEP_ORDER.length - 1];

export const selectCanProceed = (state: WorkflowState): boolean =>
  state.steps[state.currentStep].status === 'completed';

export const selectStepLabel = (step: WorkflowStep): string => {
  const labels: Record<WorkflowStep, string> = {
    import: 'Import',
    commitments: 'Commitments',
    policy: 'Policy',
    manifest: 'Manifest',
    proof: 'Proof',
    export: 'Export',
  };
  return labels[step];
};

export { STEP_ORDER };
