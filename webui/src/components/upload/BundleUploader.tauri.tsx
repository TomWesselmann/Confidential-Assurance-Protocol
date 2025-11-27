/**
 * CAP Bundle Uploader Component (Tauri Version)
 *
 * @description File selector for proof bundles using Tauri dialog API
 * @architecture Imperative Shell (React UI Component)
 * @offline Completely offline - no network requests
 */

import { useState } from 'react';
import { selectBundleFile, verifyBundle } from '../../lib/tauri';
import { useVerificationStore } from '../../store/verificationStore';

export const BundleUploader: React.FC = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { setVerificationResult, setVerificationError } =
    useVerificationStore();

  const handleSelectBundle = async () => {
    try {
      setIsLoading(true);
      setError(null);
      setVerificationError(null);

      // Open Tauri file dialog
      const bundlePath = await selectBundleFile();

      if (!bundlePath) {
        // User cancelled
        setIsLoading(false);
        return;
      }

      // Verify bundle immediately (offline)
      const result = await verifyBundle({
        bundlePath,
        options: {
          checkTimestamp: false, // Offline mode
          checkRegistry: false, // Offline mode
        },
      });

      // Store verification result in global state
      setVerificationResult(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      setVerificationError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="text-center">
      <svg
        xmlns="http://www.w3.org/2000/svg"
        fill="none"
        viewBox="0 0 24 24"
        strokeWidth={1.5}
        stroke="currentColor"
        className="w-3 h-3 mx-auto text-gray-400 mb-1"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z"
        />
      </svg>

      {isLoading ? (
        <div className="space-y-2">
          <p className="text-xs text-gray-600 dark:text-gray-400">Verifiziere...</p>
          <div className="w-24 h-1 mx-auto bg-gray-200 rounded-full overflow-hidden">
            <div className="h-full bg-blue-500 animate-pulse" style={{ width: '60%' }} />
          </div>
        </div>
      ) : (
        <>
          <p className="text-xs text-gray-500 dark:text-gray-400 mb-3">
            ZIP-Datei zur Offline-Verifikation
          </p>
          <button
            onClick={handleSelectBundle}
            disabled={isLoading}
            className="px-4 py-2 bg-blue-600 text-white text-sm rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors font-medium"
          >
            Bundle auswählen
          </button>
        </>
      )}

      {error && (
        <div className="mt-3 p-2 bg-red-50 dark:bg-red-900/20 rounded text-xs text-red-600 dark:text-red-400">
          {error}
        </div>
      )}
    </div>
  );
};
