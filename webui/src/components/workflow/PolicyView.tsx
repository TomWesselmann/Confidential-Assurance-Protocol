/**
 * Policy View Component
 *
 * @description Step 3 - Load and validate policy file
 */

import { useState } from 'react';
import { useWorkflowStore } from '../../store/workflowStore';
import { selectPolicyFile, loadPolicy, truncateHash, readPolicyContent } from '../../lib/tauri';

export const PolicyView: React.FC = () => {
  const {
    projectPath,
    policyInfo,
    setPolicyInfo,
    setStepStatus,
    goToNextStep,
    goToPreviousStep,
  } = useWorkflowStore();

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showDetails, setShowDetails] = useState(false);
  const [policyContent, setPolicyContent] = useState<string | null>(null);
  const [loadingContent, setLoadingContent] = useState(false);
  const [contentError, setContentError] = useState<string | null>(null);

  const handleLoadPolicy = async () => {
    if (!projectPath) return;

    try {
      setIsLoading(true);
      setError(null);
      setStepStatus('policy', 'in_progress');

      const filePath = await selectPolicyFile();
      if (!filePath) {
        setIsLoading(false);
        setStepStatus('policy', 'pending');
        return;
      }

      const result = await loadPolicy(projectPath, filePath);
      setPolicyInfo(result);
      setPolicyContent(null);
      setShowDetails(false);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      setStepStatus('policy', 'error', errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handleShowDetails = async () => {
    if (!projectPath) return;
    if (showDetails) { setShowDetails(false); return; }
    if (!policyContent) {
      try {
        setLoadingContent(true);
        setContentError(null);
        const content = await readPolicyContent(projectPath);
        setPolicyContent(content);
        setShowDetails(true);
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        console.error('Failed to load policy content:', errorMsg);
        setContentError(errorMsg);
        setShowDetails(true);
      } finally {
        setLoadingContent(false);
      }
    } else {
      setShowDetails(true);
    }
  };

  const isComplete = policyInfo !== null;

  return (
    <div className="space-y-3">
      <div className="text-center">
        <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100">Select Policy</h2>
        <p className="text-xs text-gray-500">YAML file with compliance rules</p>
      </div>

      <div className="max-w-sm mx-auto">
        <div className={`border rounded p-3 text-center bg-white dark:bg-gray-800 ${isComplete ? 'border-green-500' : 'border-gray-200 dark:border-gray-700'}`}>
          {!isComplete ? (
            <>
              <svg className="w-8 h-8 mx-auto text-gray-400 mb-2" width="32" height="32" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <p className="text-xs text-gray-500 mb-2">Define compliance checks</p>
              <button
                onClick={handleLoadPolicy}
                disabled={isLoading}
                className="px-3 py-1.5 bg-blue-600 text-white text-xs rounded font-medium hover:bg-blue-700 disabled:opacity-50"
              >
                {isLoading ? 'Loading...' : 'Select Policy'}
              </button>
            </>
          ) : (
            <>
              <div className="flex items-center justify-center mb-2">
                <svg className="w-4 h-4 flex-shrink-0 text-green-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span className="ml-1 text-xs font-semibold text-green-600">Policy loaded</span>
              </div>
              <div className="space-y-1 text-left bg-gray-50 dark:bg-gray-900 rounded p-2 text-[10px]">
                <div className="flex justify-between"><span className="text-gray-500">Name:</span><span className="font-medium">{policyInfo.name}</span></div>
                <div className="flex justify-between"><span className="text-gray-500">Version:</span><span className="font-mono">{policyInfo.version}</span></div>
                <div className="flex justify-between"><span className="text-gray-500">Rules:</span><span>{policyInfo.rules_count}</span></div>
                <div className="flex justify-between border-t border-gray-200 pt-1"><span className="text-gray-500">Hash:</span><code className="font-mono text-blue-600" title={policyInfo.hash}>{truncateHash(policyInfo.hash, 6)}</code></div>
              </div>
              <div className="flex gap-2 mt-2">
                <button onClick={handleShowDetails} disabled={loadingContent} className="flex-1 px-2 py-1 text-[10px] text-blue-600 bg-blue-50 rounded hover:bg-blue-100">
                  {loadingContent ? 'Loading...' : showDetails ? 'Hide' : 'Details'}
                </button>
                <button onClick={handleLoadPolicy} disabled={isLoading} className="flex-1 px-2 py-1 text-[10px] text-gray-600 bg-gray-100 rounded hover:bg-gray-200">Change</button>
              </div>
              {showDetails && (
                <div className="mt-2 text-left">
                  {contentError ? (
                    <div className="p-2 bg-red-50 rounded text-[10px] text-red-600">
                      Error: {contentError}
                    </div>
                  ) : policyContent ? (
                    <pre className="p-2 bg-gray-900 rounded text-[10px] text-green-400 font-mono overflow-auto max-h-32 whitespace-pre-wrap">{policyContent}</pre>
                  ) : (
                    <div className="p-2 bg-gray-100 rounded text-[10px] text-gray-500">
                      No content available
                    </div>
                  )}
                </div>
              )}
            </>
          )}
          {error && <div className="mt-2 p-1.5 bg-red-50 rounded text-[10px] text-red-600">{error}</div>}
        </div>
      </div>

      <div className="flex justify-between max-w-sm mx-auto pt-1">
        <button onClick={goToPreviousStep} className="px-3 py-1.5 rounded text-xs font-medium text-gray-600 bg-gray-100 hover:bg-gray-200">Back</button>
        <button
          onClick={goToNextStep}
          disabled={!isComplete}
          className={`px-3 py-1.5 rounded text-xs font-medium ${isComplete ? 'bg-blue-600 text-white hover:bg-blue-700' : 'bg-gray-300 text-gray-500 cursor-not-allowed'}`}
        >
          Next
        </button>
      </div>
    </div>
  );
};
