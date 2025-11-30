/**
 * CAP Desktop Prover - Main Application (Tauri Version)
 *
 * @description Streamlined workflow: Start proof -> Work -> Save bundle at end
 * @architecture Imperative Shell (React UI)
 * @offline Completely offline - no API config, no network requests
 */

import { useState, useEffect } from 'react';
import { BundleUploader } from './components/upload/BundleUploader.tauri';
import { VerificationView } from './components/verification/VerificationView';
import { useVerificationStore } from './store/verificationStore';
import { useWorkflowStore } from './store/workflowStore';
import {
  WorkflowStepper,
  ImportView,
  CommitmentsView,
  PolicyView,
  ManifestView,
  ProofView,
  ExportView,
} from './components/workflow';
import { AuditTimeline } from './components/audit/AuditTimeline';
import {
  createTempProject,
  createProjectInFolder,
  selectWorkingFolder,
} from './lib/tauri';

type AppMode = 'proofer' | 'verifier';

function App() {
  const [mode, setMode] = useState<AppMode>('proofer');
  const [isCreating, setIsCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showFolderOption, setShowFolderOption] = useState(false);
  const [customFolder, setCustomFolder] = useState<string | null>(null);

  // Verification store (for Verifier mode)
  const { verificationResult, reset: resetVerification } = useVerificationStore();

  // Workflow store (for Prover mode)
  const { projectPath, projectName, currentStep, steps, setProject, reset: resetWorkflow } =
    useWorkflowStore();

  // Reset when switching modes
  useEffect(() => {
    if (mode === 'proofer') {
      resetVerification();
    }
  }, [mode, resetVerification]);

  // Start a new proof workflow
  async function handleStartProof() {
    setIsCreating(true);
    setError(null);

    try {
      let project;
      if (customFolder) {
        // Use custom folder
        project = await createProjectInFolder(customFolder);
      } else {
        // Use temp folder (default)
        project = await createTempProject();
      }
      setProject(project.path, project.name);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error creating project');
    } finally {
      setIsCreating(false);
    }
  }

  // Select custom working folder
  async function handleSelectFolder() {
    try {
      const folder = await selectWorkingFolder();
      if (folder) {
        setCustomFolder(folder);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error selecting folder');
    }
  }

  // Clear custom folder selection
  function handleClearFolder() {
    setCustomFolder(null);
  }

  // Render current workflow step
  const renderWorkflowStep = () => {
    switch (currentStep) {
      case 'import':
        return <ImportView />;
      case 'commitments':
        return <CommitmentsView />;
      case 'policy':
        return <PolicyView />;
      case 'manifest':
        return <ManifestView />;
      case 'proof':
        return <ProofView />;
      case 'export':
        return <ExportView />;
      default:
        return <ImportView />;
    }
  };

  return (
    <div className="h-screen flex flex-col bg-gray-50 dark:bg-gray-900 overflow-hidden">
      {/* Header */}
      <header className="flex-shrink-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 shadow-sm">
        <div className="px-4 py-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              <h1 className="text-lg font-bold text-gray-900 dark:text-gray-100">
                CAP Desktop Prover
              </h1>
              <span className="text-xs text-gray-400 dark:text-gray-500 bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded">
                Offline
              </span>
            </div>

            {/* Mode Tabs */}
            <div className="flex items-center space-x-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-0.5">
              <button
                onClick={() => setMode('proofer')}
                className={`
                  px-3 py-1.5 text-sm font-medium rounded transition-colors
                  ${
                    mode === 'proofer'
                      ? 'bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm'
                      : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
                  }
                `}
              >
                Prover
              </button>
              <button
                onClick={() => setMode('verifier')}
                className={`
                  px-3 py-1.5 text-sm font-medium rounded transition-colors
                  ${
                    mode === 'verifier'
                      ? 'bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm'
                      : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
                  }
                `}
              >
                Verifier
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="flex-1 overflow-auto">
        <div className="max-w-5xl mx-auto px-4 py-6 h-full">
          {mode === 'proofer' ? (
            // PROOFER MODE
            <>
              {!projectPath ? (
                // No project - show start screen
                <div className="h-full flex items-center justify-center">
                  <div className="text-center max-w-md">
                    {/* Icon */}
                    <div className="w-16 h-16 mx-auto mb-4 bg-blue-100 dark:bg-blue-900/30 rounded-full flex items-center justify-center">
                      <svg
                        className="w-8 h-8 text-blue-600 dark:text-blue-400"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={1.5}
                          d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
                        />
                      </svg>
                    </div>

                    {/* Title */}
                    <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-200 mb-2">
                      Create Compliance Proof
                    </h2>
                    <p className="text-sm text-gray-500 dark:text-gray-400 mb-6">
                      6-step workflow: Import data, apply policy, generate proof
                    </p>

                    {/* Error */}
                    {error && (
                      <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-sm text-red-600 dark:text-red-400">
                        {error}
                      </div>
                    )}

                    {/* Start Button */}
                    <button
                      onClick={handleStartProof}
                      disabled={isCreating}
                      className="w-full px-6 py-3 bg-blue-600 text-white text-base font-medium rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors mb-3"
                    >
                      {isCreating ? (
                        <span className="flex items-center justify-center gap-2">
                          <svg className="animate-spin h-5 w-5" width="20" height="20" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                          </svg>
                          Creating project...
                        </span>
                      ) : (
                        'Start New Proof'
                      )}
                    </button>

                    {/* Folder Option Toggle */}
                    <button
                      onClick={() => setShowFolderOption(!showFolderOption)}
                      className="text-sm text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-300"
                    >
                      {showFolderOption ? 'Hide options' : 'Choose working folder (optional)'}
                    </button>

                    {/* Folder Selection */}
                    {showFolderOption && (
                      <div className="mt-4 p-4 bg-gray-50 dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
                        <div className="text-xs text-gray-500 dark:text-gray-400 mb-2">
                          Working folder for temporary files:
                        </div>
                        {customFolder ? (
                          <div className="flex items-center gap-2">
                            <div className="flex-1 text-sm font-mono text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-700 px-3 py-2 rounded border truncate">
                              {customFolder}
                            </div>
                            <button
                              onClick={handleClearFolder}
                              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                              title="Reset"
                            >
                              <svg className="w-5 h-5" width="20" height="20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                              </svg>
                            </button>
                          </div>
                        ) : (
                          <button
                            onClick={handleSelectFolder}
                            className="w-full text-sm text-blue-600 hover:text-blue-700 dark:text-blue-400 bg-white dark:bg-gray-700 px-3 py-2 rounded border border-dashed border-gray-300 dark:border-gray-600 hover:border-blue-400"
                          >
                            Select folder...
                          </button>
                        )}
                        <div className="text-xs text-gray-400 dark:text-gray-500 mt-2">
                          Default: Temporary system folder (saved as bundle at the end)
                        </div>
                      </div>
                    )}
                  </div>
                </div>
              ) : (
                // Project exists - show workflow
                <div className="space-y-4 h-full flex flex-col">
                  {/* Project Info Bar */}
                  <div className="flex-shrink-0 flex items-center justify-between bg-white dark:bg-gray-800 rounded-lg px-4 py-3 border border-gray-200 dark:border-gray-700 shadow-sm">
                    <div className="flex items-center space-x-3">
                      <div className="w-8 h-8 bg-blue-100 dark:bg-blue-900/30 rounded-lg flex items-center justify-center">
                        <svg
                          className="w-4 h-4 text-blue-600 dark:text-blue-400"
                          fill="currentColor"
                          viewBox="0 0 20 20"
                        >
                          <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                        </svg>
                      </div>
                      <div>
                        <div className="text-sm font-medium text-gray-900 dark:text-gray-100">
                          {projectName}
                        </div>
                        <div className="text-xs text-gray-400 dark:text-gray-500 truncate max-w-xs" title={projectPath}>
                          {projectPath.replace(/^\/Users\/[^/]+/, '~')}
                        </div>
                      </div>
                    </div>
                    <button
                      onClick={resetWorkflow}
                      className="text-sm text-gray-500 hover:text-red-600 dark:text-gray-400 dark:hover:text-red-400 transition-colors"
                    >
                      Cancel
                    </button>
                  </div>

                  {/* Workflow Stepper */}
                  <div className="flex-shrink-0">
                    <WorkflowStepper />
                  </div>

                  {/* Current Step Content */}
                  <div className="flex-1 overflow-auto">
                    {renderWorkflowStep()}
                  </div>

                  {/* Audit Log (collapsible) */}
                  {currentStep !== 'export' && (
                    <details className="flex-shrink-0 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700">
                      <summary className="px-4 py-2 cursor-pointer text-sm text-gray-600 dark:text-gray-400 hover:text-gray-800 dark:hover:text-gray-200">
                        Show Audit Log
                      </summary>
                      <div className="px-4 pb-4 max-h-48 overflow-auto">
                        <AuditTimeline
                          projectPath={projectPath}
                          refreshKey={`${currentStep}-${Object.values(steps).map(s => s.status).join('-')}`}
                        />
                      </div>
                    </details>
                  )}
                </div>
              )}
            </>
          ) : (
            // VERIFIER MODE
            <div className="h-full flex flex-col">
              {!verificationResult ? (
                <div className="flex-1 flex items-center justify-center">
                  <BundleUploader />
                </div>
              ) : (
                <div className="flex-1 flex flex-col space-y-4 overflow-auto">
                  <div className="flex-shrink-0 flex justify-end">
                    <button
                      onClick={resetVerification}
                      className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
                    >
                      Verify New Bundle
                    </button>
                  </div>
                  <div className="flex-1 overflow-auto">
                    <VerificationView />
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </main>

      {/* Footer */}
      <footer className="flex-shrink-0 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700 px-4 py-2">
        <div className="flex items-center justify-between text-xs text-gray-400 dark:text-gray-500">
          <span>CAP Desktop Prover v0.12.0</span>
          <span>Tauri 2.0 | Offline-First</span>
        </div>
      </footer>
    </div>
  );
}

export default App;
