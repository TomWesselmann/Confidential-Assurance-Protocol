/**
 * Manifest View Component
 *
 * @description Step 4 - Build manifest from commitments and policy
 */

import { useState } from 'react';
import { useWorkflowStore } from '../../store/workflowStore';
import { buildManifest, truncateHash, readManifestContent } from '../../lib/tauri';

export const ManifestView: React.FC = () => {
  const {
    projectPath,
    manifestResult,
    setManifestResult,
    setStepStatus,
    goToNextStep,
    goToPreviousStep,
  } = useWorkflowStore();

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showDetails, setShowDetails] = useState(false);
  const [manifestContent, setManifestContent] = useState<string | null>(null);
  const [loadingContent, setLoadingContent] = useState(false);

  const handleBuildManifest = async () => {
    if (!projectPath) return;
    try {
      setIsLoading(true);
      setError(null);
      setStepStatus('manifest', 'in_progress');
      const result = await buildManifest(projectPath);
      setManifestResult(result);
      setManifestContent(null);
      setShowDetails(false);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unbekannter Fehler';
      setError(errorMessage);
      setStepStatus('manifest', 'error', errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handleShowDetails = async () => {
    if (!projectPath) return;
    if (showDetails) { setShowDetails(false); return; }
    if (!manifestContent) {
      try {
        setLoadingContent(true);
        const content = await readManifestContent(projectPath);
        const parsed = JSON.parse(content);
        setManifestContent(JSON.stringify(parsed, null, 2));
      } catch (err) {
        console.error('Failed to load manifest content:', err);
      } finally {
        setLoadingContent(false);
      }
    }
    setShowDetails(true);
  };

  const isComplete = manifestResult !== null;

  return (
    <div className="space-y-3">
      <div className="text-center">
        <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100">Manifest erstellen</h2>
        <p className="text-xs text-gray-500">Commitments + Policy zusammenfassen</p>
      </div>

      <div className="max-w-sm mx-auto">
        <div className={`border rounded p-3 text-center bg-white dark:bg-gray-800 ${isComplete ? 'border-green-500' : 'border-gray-200 dark:border-gray-700'}`}>
          {!isComplete ? (
            <>
              <svg className="w-8 h-8 mx-auto text-gray-400 mb-2" width="32" height="32" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
              </svg>
              <p className="text-xs text-gray-500 mb-2">Hashes und Audit-Metadaten</p>
              <button
                onClick={handleBuildManifest}
                disabled={isLoading}
                className="px-3 py-1.5 bg-blue-600 text-white text-xs rounded font-medium hover:bg-blue-700 disabled:opacity-50"
              >
                {isLoading ? 'Erstelle...' : 'Manifest erstellen'}
              </button>
            </>
          ) : (
            <>
              <div className="flex items-center justify-center mb-2">
                <svg className="w-4 h-4 flex-shrink-0 text-green-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span className="ml-1 text-xs font-semibold text-green-600">Manifest erstellt</span>
              </div>
              <div className="space-y-1 text-left bg-gray-50 dark:bg-gray-900 rounded p-2 text-[10px]">
                <div className="flex justify-between"><span className="text-gray-600 font-medium">Hash:</span><code className="font-mono text-blue-600" title={manifestResult.manifest_hash}>{truncateHash(manifestResult.manifest_hash, 6)}</code></div>
                <div className="border-t border-gray-200 pt-1 space-y-0.5">
                  <div className="flex justify-between"><span className="text-gray-500">Supplier:</span><code className="font-mono text-gray-500">{truncateHash(manifestResult.supplier_root, 4)}</code></div>
                  <div className="flex justify-between"><span className="text-gray-500">UBO:</span><code className="font-mono text-gray-500">{truncateHash(manifestResult.ubo_root, 4)}</code></div>
                  <div className="flex justify-between"><span className="text-gray-500">Policy:</span><code className="font-mono text-gray-500">{truncateHash(manifestResult.policy_hash, 4)}</code></div>
                </div>
              </div>
              <button onClick={handleShowDetails} disabled={loadingContent} className="mt-2 w-full px-2 py-1 text-[10px] text-blue-600 bg-blue-50 rounded hover:bg-blue-100">
                {loadingContent ? 'Lade...' : showDetails ? 'Ausblenden' : 'Details anzeigen'}
              </button>
              {showDetails && manifestContent && (
                <div className="mt-2 text-left">
                  <pre className="p-2 bg-gray-900 rounded text-[10px] text-green-400 font-mono overflow-auto max-h-32 whitespace-pre-wrap">{manifestContent}</pre>
                </div>
              )}
            </>
          )}
          {error && <div className="mt-2 p-1.5 bg-red-50 rounded text-[10px] text-red-600">{error}</div>}
        </div>
      </div>

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
