/**
 * Proof View Component
 *
 * @description Step 5 - Generate cryptographic proof with progress tracking
 */

import { useState } from 'react';
import { useWorkflowStore } from '../../store/workflowStore';
import { buildProof, truncateHash } from '../../lib/tauri';

export const ProofView: React.FC = () => {
  const {
    projectPath,
    proofResult,
    proofProgress,
    setProofResult,
    setProofProgress,
    setStepStatus,
    goToNextStep,
    goToPreviousStep,
  } = useWorkflowStore();

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleBuildProof = async () => {
    if (!projectPath) return;
    try {
      setIsLoading(true);
      setError(null);
      setStepStatus('proof', 'in_progress');
      setProofProgress({ percent: 0, message: 'Initialisiere...' });
      const result = await buildProof(projectPath, (progress) => {
        setProofProgress(progress);
      });
      setProofResult(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unbekannter Fehler';
      setError(errorMessage);
      setStepStatus('proof', 'error', errorMessage);
      setProofProgress(null);
    } finally {
      setIsLoading(false);
    }
  };

  const isComplete = proofResult !== null;

  return (
    <div className="space-y-3">
      <div className="text-center">
        <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100">Proof generieren</h2>
        <p className="text-xs text-gray-500">Kryptographischen Beweis erstellen</p>
      </div>

      <div className="max-w-sm mx-auto">
        <div className={`border rounded p-3 text-center bg-white dark:bg-gray-800 ${isComplete ? 'border-green-500' : isLoading ? 'border-blue-500' : 'border-gray-200 dark:border-gray-700'}`}>
          {!isComplete && !isLoading ? (
            <>
              <svg className="w-8 h-8 mx-auto text-gray-400 mb-2" width="32" height="32" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
              </svg>
              <p className="text-xs text-gray-500 mb-2">Proof-Generator prüft Constraints</p>
              <button
                onClick={handleBuildProof}
                className="px-3 py-1.5 bg-blue-600 text-white text-xs rounded font-medium hover:bg-blue-700"
              >
                Proof generieren
              </button>
            </>
          ) : isLoading ? (
            <>
              <svg className="w-8 h-8 mx-auto text-blue-500 animate-pulse mb-2" width="32" height="32" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
              </svg>
              <p className="text-xs text-blue-600 mb-2">Generiere Proof...</p>
              <div className="max-w-[200px] mx-auto">
                <div className="flex justify-between text-[10px] text-gray-500 mb-1">
                  <span>{proofProgress?.message || 'Verarbeite...'}</span>
                  <span>{proofProgress?.percent || 0}%</span>
                </div>
                <div className="w-full h-1.5 bg-gray-200 rounded-full overflow-hidden">
                  <div className="h-full bg-blue-500 transition-all duration-300" style={{ width: `${proofProgress?.percent || 0}%` }} />
                </div>
              </div>
            </>
          ) : (
            <>
              <div className="flex items-center justify-center mb-2">
                <svg className="w-4 h-4 flex-shrink-0 text-green-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span className="ml-1 text-xs font-semibold text-green-600">Proof generiert</span>
              </div>
              {proofResult && (
                <div className="space-y-1 text-left bg-gray-50 dark:bg-gray-900 rounded p-2 text-[10px]">
                  <div className="flex justify-between"><span className="text-gray-600 font-medium">Hash:</span><code className="font-mono text-blue-600" title={proofResult.proof_hash}>{truncateHash(proofResult.proof_hash, 6)}</code></div>
                  <div className="flex justify-between"><span className="text-gray-500">Backend:</span><span className="font-mono">{proofResult.backend}</span></div>
                </div>
              )}
            </>
          )}
          {error && (
            <div className="mt-2 p-1.5 bg-red-50 rounded text-[10px] text-red-600">
              {error}
              <button onClick={handleBuildProof} className="ml-2 underline hover:no-underline">Erneut</button>
            </div>
          )}
        </div>
      </div>

      <div className="flex justify-between max-w-sm mx-auto pt-1">
        <button onClick={goToPreviousStep} disabled={isLoading} className="px-3 py-1.5 rounded text-xs font-medium text-gray-600 bg-gray-100 hover:bg-gray-200 disabled:opacity-50">Zurück</button>
        <button
          onClick={goToNextStep}
          disabled={!isComplete || isLoading}
          className={`px-3 py-1.5 rounded text-xs font-medium ${isComplete && !isLoading ? 'bg-blue-600 text-white hover:bg-blue-700' : 'bg-gray-300 text-gray-500 cursor-not-allowed'}`}
        >
          Weiter
        </button>
      </div>
    </div>
  );
};
