/**
 * CAP Verifier WebUI - Main Application (Tauri Version)
 *
 * @description Main application component for offline desktop proof verification
 * @architecture Imperative Shell (React UI)
 * @offline Completely offline - no API config, no network requests
 */

import { BundleUploader } from './components/upload/BundleUploader';
import { VerificationView } from './components/verification/VerificationView';
import { useVerificationStore } from './store/verificationStore';

function App() {
  const { verificationResult, reset } = useVerificationStore();

  const handleReset = () => {
    reset();
  };

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700 shadow-sm">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div>
              <h1 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                CAP Desktop Proofer
              </h1>
              <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                Offline Proof Verification • Confidential Assurance Protocol
              </p>
            </div>

            {verificationResult && (
              <button
                onClick={handleReset}
                className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
              >
                Neues Bundle
              </button>
            )}
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-6 py-8">
        {/* Upload Section */}
        {!verificationResult && <BundleUploader />}

        {/* Verification Results */}
        {verificationResult && (
          <div className="space-y-8">
            <VerificationView />
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="mt-16 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <p className="text-center text-sm text-gray-500 dark:text-gray-400">
            CAP Desktop Proofer v0.1.0 • Tauri 2.0 • Offline-First Architecture
          </p>
        </div>
      </footer>
    </div>
  );
}

export default App;
