/**
 * Export View Component
 *
 * @description Step 6 - Export bundle as ZIP file
 */

import { useState } from 'react';
import { useWorkflowStore } from '../../store/workflowStore';
import { selectExportPath, exportBundle, truncateHash, formatFileSize } from '../../lib/tauri';

export const ExportView: React.FC = () => {
  const {
    projectPath,
    exportResult,
    setExportResult,
    setStepStatus,
    goToPreviousStep,
    reset,
  } = useWorkflowStore();

  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleExport = async () => {
    if (!projectPath) return;
    try {
      setIsLoading(true);
      setError(null);
      setStepStatus('export', 'in_progress');
      const outputPath = await selectExportPath();
      if (!outputPath) {
        setIsLoading(false);
        setStepStatus('export', 'pending');
        return;
      }
      const result = await exportBundle(projectPath, outputPath);
      setExportResult(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      setStepStatus('export', 'error', errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const isComplete = exportResult !== null;

  return (
    <div className="space-y-3">
      <div className="text-center">
        <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100">Export Bundle</h2>
        <p className="text-xs text-gray-500">Save proof bundle as ZIP</p>
      </div>

      <div className="max-w-sm mx-auto">
        <div className={`border rounded p-3 text-center bg-white dark:bg-gray-800 ${isComplete ? 'border-green-500' : 'border-gray-200 dark:border-gray-700'}`}>
          {!isComplete ? (
            <>
              <svg className="w-8 h-8 mx-auto text-gray-400 mb-2" width="32" height="32" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
              </svg>
              <p className="text-xs text-gray-500 mb-2">Manifest, proof and metadata</p>
              <button
                onClick={handleExport}
                disabled={isLoading}
                className="px-3 py-1.5 bg-green-600 text-white text-xs rounded font-medium hover:bg-green-700 disabled:opacity-50"
              >
                {isLoading ? 'Exporting...' : 'Export Bundle'}
              </button>
            </>
          ) : (
            <>
              <div className="flex items-center justify-center mb-2">
                <svg className="w-4 h-4 flex-shrink-0 text-green-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span className="ml-1 text-xs font-semibold text-green-600">Export successful!</span>
              </div>
              <div className="space-y-1 text-left bg-gray-50 dark:bg-gray-900 rounded p-2 text-[10px]">
                <div className="flex justify-between"><span className="text-gray-500">Size:</span><span className="font-medium">{formatFileSize(exportResult.size_bytes)}</span></div>
                <div className="flex justify-between"><span className="text-gray-500">Hash:</span><code className="font-mono text-blue-600" title={exportResult.hash}>{truncateHash(exportResult.hash, 6)}</code></div>
                <div className="flex justify-between"><span className="text-gray-500">Files:</span><span>{exportResult.files.length}</span></div>
                <div className="border-t border-gray-200 pt-1 mt-1 flex flex-wrap gap-1">
                  {exportResult.files.map((file, i) => (
                    <span key={i} className="px-1 py-0.5 bg-gray-200 dark:bg-gray-700 rounded text-gray-600 font-mono">{file}</span>
                  ))}
                </div>
              </div>
              <div className="flex gap-2 mt-2">
                <button onClick={handleExport} className="flex-1 px-2 py-1 text-[10px] text-gray-600 bg-gray-100 rounded hover:bg-gray-200">Re-export</button>
                <button onClick={reset} className="flex-1 px-2 py-1 text-[10px] text-blue-600 bg-blue-50 rounded hover:bg-blue-100">New Project</button>
              </div>
            </>
          )}
          {error && <div className="mt-2 p-1.5 bg-red-50 rounded text-[10px] text-red-600">{error}</div>}
        </div>
      </div>

      <div className="flex justify-between max-w-sm mx-auto pt-1">
        <button onClick={goToPreviousStep} className="px-3 py-1.5 rounded text-xs font-medium text-gray-600 bg-gray-100 hover:bg-gray-200">Back</button>
        {isComplete && (
          <div className="text-green-600 flex items-center text-xs">
            <svg className="w-4 h-4 flex-shrink-0" width="16" height="16" fill="currentColor" viewBox="0 0 20 20">
              <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
            </svg>
            <span className="ml-1 font-medium">Done!</span>
          </div>
        )}
      </div>
    </div>
  );
};
