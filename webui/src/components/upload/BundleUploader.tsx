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

  const { setVerificationResult, setVerificationError, setIsVerifying } =
    useVerificationStore();

  const handleSelectBundle = async () => {
    try {
      setIsLoading(true);
      setIsVerifying(true);
      setError(null);
      setVerificationError(null);

      // Open Tauri file dialog
      const bundlePath = await selectBundleFile();

      if (!bundlePath) {
        // User cancelled
        setIsLoading(false);
        setIsVerifying(false);
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
    <div className="w-full max-w-2xl mx-auto p-6">
      <div className="border-2 border-dashed rounded-lg p-12 text-center border-gray-300 dark:border-gray-600 hover:border-blue-400 transition-colors duration-200">
        <div className="space-y-4">
          {/* Icon */}
          <div className="text-6xl text-gray-400">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={1.5}
              stroke="currentColor"
              className="w-16 h-16 mx-auto"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z"
              />
            </svg>
          </div>

          {/* Loading State */}
          {isLoading ? (
            <div className="space-y-2">
              <p className="text-lg font-semibold text-gray-700 dark:text-gray-300">
                Verifying bundle...
              </p>
              <div className="w-48 h-2 mx-auto bg-gray-200 rounded-full overflow-hidden">
                <div
                  className="h-full bg-blue-500 animate-pulse"
                  style={{ width: '60%' }}
                />
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              <div className="space-y-2">
                <p className="text-lg font-semibold text-gray-700 dark:text-gray-300">
                  Verify CAP Proof Bundle
                </p>
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  Select a bundle ZIP file for offline verification
                </p>
                <p className="text-xs text-gray-400 dark:text-gray-500">
                  Completely offline - no network requests
                </p>
              </div>

              {/* Select Button */}
              <button
                onClick={handleSelectBundle}
                disabled={isLoading}
                className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors font-semibold"
              >
                Select Bundle
              </button>
            </div>
          )}
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="mt-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-sm font-semibold text-red-800 dark:text-red-300">
            Verification error
          </p>
          <p className="text-xs text-red-600 dark:text-red-400 mt-1">{error}</p>
        </div>
      )}
    </div>
  );
};
