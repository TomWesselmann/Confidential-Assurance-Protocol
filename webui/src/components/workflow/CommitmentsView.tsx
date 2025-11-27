/**
 * Commitments View Component
 *
 * @description Step 2 - Create cryptographic commitments (Merkle roots)
 */

import { useState } from 'react';
import { useWorkflowStore } from '../../store/workflowStore';
import { createCommitments, truncateHash } from '../../lib/tauri';

export const CommitmentsView: React.FC = () => {
  const {
    projectPath,
    commitmentsResult,
    setCommitmentsResult,
    setStepStatus,
    goToNextStep,
    goToPreviousStep,
  } = useWorkflowStore();

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleCreateCommitments = async () => {
    if (!projectPath) return;

    try {
      setIsLoading(true);
      setError(null);
      setStepStatus('commitments', 'in_progress');

      const result = await createCommitments(projectPath);
      setCommitmentsResult(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unbekannter Fehler';
      setError(errorMessage);
      setStepStatus('commitments', 'error', errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const isComplete = commitmentsResult !== null;

  return (
    <div className="space-y-3">
      {/* Header */}
      <div className="text-center">
        <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100">Commitments erstellen</h2>
        <p className="text-xs text-gray-500">Merkle-Roots generieren</p>
      </div>

      {/* Main Card */}
      <div className="max-w-sm mx-auto">
        <div className={`border rounded p-3 text-center bg-white dark:bg-gray-800 ${isComplete ? 'border-green-500' : 'border-gray-200 dark:border-gray-700'}`}>
          {!isComplete ? (
            <>
              <svg className="w-8 h-8 mx-auto text-gray-400 mb-2" width="32" height="32" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
              </svg>
              <p className="text-xs text-gray-500 mb-2">CSV-Daten zu Merkle-Bäumen</p>
              <button
                onClick={handleCreateCommitments}
                disabled={isLoading}
                className="px-3 py-1.5 bg-blue-600 text-white text-xs rounded font-medium hover:bg-blue-700 disabled:opacity-50"
              >
                {isLoading ? 'Erstelle...' : 'Commitments erstellen'}
              </button>
            </>
          ) : (
            <>
              <div className="flex items-center justify-center mb-2">
                <svg className="w-4 h-4 flex-shrink-0 text-green-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span className="ml-1 text-xs font-semibold text-green-600">Erfolgreich</span>
              </div>
              <div className="space-y-1 text-left bg-gray-50 dark:bg-gray-900 rounded p-2 text-[10px]">
                <div className="flex justify-between">
                  <span className="text-gray-500">Supplier:</span>
                  <code className="font-mono text-gray-600" title={commitmentsResult.supplier_root}>{truncateHash(commitmentsResult.supplier_root, 6)}</code>
                </div>
                <div className="flex justify-between">
                  <span className="text-gray-500">UBO:</span>
                  <code className="font-mono text-gray-600" title={commitmentsResult.ubo_root}>{truncateHash(commitmentsResult.ubo_root, 6)}</code>
                </div>
                <div className="flex justify-between border-t border-gray-200 dark:border-gray-700 pt-1">
                  <span className="text-gray-600 font-medium">Company:</span>
                  <code className="font-mono text-blue-600" title={commitmentsResult.company_root}>{truncateHash(commitmentsResult.company_root, 6)}</code>
                </div>
              </div>
            </>
          )}
          {error && <div className="mt-2 p-1.5 bg-red-50 rounded text-[10px] text-red-600">{error}</div>}
        </div>
      </div>

      {/* Navigation */}
      <div className="flex justify-between max-w-sm mx-auto pt-1">
        <button onClick={goToPreviousStep} className="px-3 py-1.5 rounded text-xs font-medium text-gray-600 bg-gray-100 hover:bg-gray-200">Zurück</button>
        <button
          onClick={goToNextStep}
          disabled={!isComplete}
          className={`px-3 py-1.5 rounded text-xs font-medium ${isComplete ? 'bg-blue-600 text-white hover:bg-blue-700' : 'bg-gray-300 text-gray-500 cursor-not-allowed'}`}
        >
          Weiter
        </button>
      </div>
    </div>
  );
};
