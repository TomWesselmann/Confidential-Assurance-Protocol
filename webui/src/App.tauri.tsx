/**
 * CAP Desktop Proofer - Main Application (Tauri Version)
 *
 * @description Main application component with Proofer, Verifier, and Audit modes
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
import { ProjectSidebar } from './components/layout/ProjectSidebar';
import { AuditTimeline } from './components/audit/AuditTimeline';
import { selectWorkspace, createProject, getProjectStatus, type ProjectInfo } from './lib/tauri';

type AppMode = 'proofer' | 'verifier' | 'audit';

function App() {
  const [mode, setMode] = useState<AppMode>('proofer');
  const [isCreatingProject, setIsCreatingProject] = useState(false);
  const [projectError, setProjectError] = useState<string | null>(null);
  const [showSidebar, setShowSidebar] = useState(true);
  const [workspacePath, setWorkspacePath] = useState<string | null>(null);

  // Verification store (for Verifier mode)
  const { verificationResult, reset: resetVerification } = useVerificationStore();

  // Workflow store (for Proofer mode)
  const { projectPath, projectName, currentStep, setProject, initializeFromStatus, reset: resetWorkflow } =
    useWorkflowStore();

  // Current project info
  const currentProject: ProjectInfo | null = projectPath
    ? { path: projectPath, name: projectName || '', createdAt: new Date().toISOString() }
    : null;

  // Handle project selection from sidebar
  const handleProjectSelect = async (project: ProjectInfo) => {
    try {
      setProjectError(null);
      // Get full project status to initialize workflow
      const status = await getProjectStatus(project.path);
      // Initialize workflow from backend status (preserves progress)
      initializeFromStatus(project.path, status.info.name, {
        hasSuppliersCSv: status.hasSuppliersCSv,
        hasUbosCsv: status.hasUbosCsv,
        hasPolicy: status.hasPolicy,
        hasCommitments: status.hasCommitments,
        hasManifest: status.hasManifest,
        hasProof: status.hasProof,
        currentStep: status.currentStep,
      });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Fehler beim Laden des Projekts';
      setProjectError(errorMessage);
    }
  };

  // Handle workspace change
  const handleWorkspaceChange = (path: string) => {
    setWorkspacePath(path);
    resetWorkflow();
  };

  // Handle project creation (quick create)
  const handleCreateProject = async () => {
    try {
      setIsCreatingProject(true);
      setProjectError(null);

      // Use current workspace or select new one
      let workspace = workspacePath;
      if (!workspace) {
        workspace = await selectWorkspace();
        if (!workspace) {
          setIsCreatingProject(false);
          return;
        }
        setWorkspacePath(workspace);
      }

      // Generate project name with timestamp
      const projectNameNew = `cap-proof-${new Date().toISOString().split('T')[0]}-${Date.now().toString(36)}`;

      // Create project
      const project = await createProject(workspace, projectNameNew);
      setProject(project.path, project.name);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unbekannter Fehler';
      setProjectError(errorMessage);
    } finally {
      setIsCreatingProject(false);
    }
  };

  // Reset when switching modes
  useEffect(() => {
    if (mode === 'proofer') {
      resetVerification();
    }
  }, [mode, resetVerification]);

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
      {/* Header - Compact */}
      <header className="flex-shrink-0 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 shadow-sm">
        <div className="px-4 py-2">
          <div className="flex items-center justify-between">
            <div className="flex items-center space-x-3">
              {/* Sidebar Toggle */}
              <button
                onClick={() => setShowSidebar(!showSidebar)}
                className="p-1 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
                title={showSidebar ? 'Sidebar ausblenden' : 'Sidebar einblenden'}
              >
                <svg className="w-5 h-5" width="20" height="20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 6h16M4 12h16M4 18h16" />
                </svg>
              </button>
              <h1 className="text-lg font-bold text-gray-900 dark:text-gray-100">
                CAP Desktop Proofer
              </h1>
              <span className="text-xs text-gray-400 dark:text-gray-500">
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
                Proofer
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
              <button
                onClick={() => setMode('audit')}
                className={`
                  px-3 py-1.5 text-sm font-medium rounded transition-colors
                  ${
                    mode === 'audit'
                      ? 'bg-white dark:bg-gray-600 text-blue-600 dark:text-blue-400 shadow-sm'
                      : 'text-gray-600 dark:text-gray-300 hover:text-gray-900 dark:hover:text-white'
                  }
                `}
              >
                Audit
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content Area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar */}
        {showSidebar && (
          <ProjectSidebar
            workspacePath={workspacePath}
            onWorkspaceChange={handleWorkspaceChange}
            currentProject={currentProject}
            onProjectSelect={handleProjectSelect}
          />
        )}

        {/* Main Content */}
        <main className="flex-1 overflow-auto">
          <div className="max-w-6xl mx-auto px-4 py-4 h-full">
            {mode === 'proofer' ? (
              // PROOFER MODE
              <>
                {!projectPath ? (
                  // No project - show create button (centered, compact)
                  <div className="h-full flex items-center justify-center">
                    <div className="text-center">
                      <svg
                        className="w-8 h-8 mx-auto text-gray-300 dark:text-gray-600 mb-2"
                        width="32"
                        height="32"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={1.5}
                          d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                        />
                      </svg>
                      <p className="text-xs text-gray-500 dark:text-gray-400 mb-3">
                        {workspacePath
                          ? 'Wählen Sie ein Projekt aus der Sidebar oder erstellen Sie ein neues'
                          : '6-Schritte Compliance-Workflow'}
                      </p>

                      {projectError && (
                        <div className="mb-3 p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-xs text-red-600 dark:text-red-400">
                          {projectError}
                        </div>
                      )}

                      <button
                        onClick={handleCreateProject}
                        disabled={isCreatingProject}
                        className="px-4 py-2 bg-blue-600 text-white text-sm rounded-lg font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors"
                      >
                        {isCreatingProject ? 'Erstelle...' : 'Neues Projekt erstellen'}
                      </button>
                    </div>
                  </div>
                ) : (
                  // Project exists - show workflow
                  <div className="space-y-4 h-full flex flex-col">
                    {/* Project Info Bar - Compact */}
                    <div className="flex-shrink-0 flex items-center justify-between bg-white dark:bg-gray-800 rounded-lg px-3 py-2 border border-gray-200 dark:border-gray-700">
                      <div className="flex items-center space-x-2">
                        <svg
                          className="w-3 h-3 text-blue-500"
                          width="12"
                          height="12"
                          fill="currentColor"
                          viewBox="0 0 20 20"
                        >
                          <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                        </svg>
                        <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                          {projectName}
                        </span>
                      </div>
                      <button
                        onClick={resetWorkflow}
                        className="text-xs text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200"
                      >
                        Schließen
                      </button>
                    </div>

                    {/* Workflow Stepper */}
                    <div className="flex-shrink-0">
                      <WorkflowStepper />
                    </div>

                    {/* Current Step Content - Scrollable if needed */}
                    <div className="flex-1 overflow-auto">
                      {renderWorkflowStep()}
                    </div>
                  </div>
                )}
              </>
            ) : mode === 'verifier' ? (
              // VERIFIER MODE
              <div className="h-full flex flex-col">
                {!verificationResult ? (
                  <div className="flex-1 flex items-center justify-center">
                    <BundleUploader />
                  </div>
                ) : (
                  <div className="flex-1 flex flex-col space-y-3 overflow-auto">
                    <div className="flex-shrink-0 flex justify-end">
                      <button
                        onClick={resetVerification}
                        className="px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
                      >
                        Neues Bundle
                      </button>
                    </div>
                    <div className="flex-1 overflow-auto">
                      <VerificationView />
                    </div>
                  </div>
                )}
              </div>
            ) : (
              // AUDIT MODE
              <div className="h-full flex flex-col">
                {!projectPath ? (
                  <div className="h-full flex items-center justify-center">
                    <div className="text-center">
                      <svg
                        className="w-8 h-8 mx-auto text-gray-300 dark:text-gray-600 mb-2"
                        width="32"
                        height="32"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={1.5}
                          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                        />
                      </svg>
                      <p className="text-xs text-gray-500 dark:text-gray-400">
                        Wählen Sie ein Projekt aus der Sidebar, um das Audit-Log anzuzeigen
                      </p>
                    </div>
                  </div>
                ) : (
                  <div className="space-y-4 h-full flex flex-col">
                    {/* Project Info Bar */}
                    <div className="flex-shrink-0 flex items-center justify-between bg-white dark:bg-gray-800 rounded-lg px-3 py-2 border border-gray-200 dark:border-gray-700">
                      <div className="flex items-center space-x-2">
                        <svg
                          className="w-3 h-3 text-purple-500"
                          width="12"
                          height="12"
                          fill="currentColor"
                          viewBox="0 0 20 20"
                        >
                          <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
                          <path fillRule="evenodd" d="M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z" clipRule="evenodd" />
                        </svg>
                        <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                          Audit-Log: {projectName}
                        </span>
                      </div>
                    </div>

                    {/* Audit Timeline */}
                    <div className="flex-1 overflow-auto bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 p-4">
                      <AuditTimeline projectPath={projectPath} />
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>
        </main>
      </div>
    </div>
  );
}

export default App;
