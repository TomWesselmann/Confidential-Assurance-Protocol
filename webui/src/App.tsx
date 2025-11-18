/**
 * CAP Verifier WebUI - Main Application
 *
 * @description Main application component orchestrating the verification workflow
 * @architecture Imperative Shell (React UI)
 */

import { useState, useEffect } from 'react';
import { BundleUploader } from './components/upload/BundleUploader';
import { ManifestViewer } from './components/manifest/ManifestViewer';
import { VerificationView } from './components/verification/VerificationView';
import { useVerificationStore } from './store/verificationStore';
import { capApiClient } from './core/api/client';

function App() {
  const {
    manifest,
    policyHash,
    proofBundle,
    setIsVerifying,
    setVerificationResult,
    setVerificationError,
    reset,
  } = useVerificationStore();

  const [apiUrl, setApiUrl] = useState('http://localhost:8080');
  const [bearerToken, setBearerToken] = useState('admin-tom');

  // Set default token on mount
  useEffect(() => {
    capApiClient.setBaseURL(apiUrl);
    capApiClient.setBearerToken(bearerToken);
  }, [apiUrl, bearerToken]);

  const handleVerify = async () => {
    if (!manifest || !policyHash) {
      setVerificationError('Manifest oder Policy Hash fehlt');
      return;
    }

    setIsVerifying(true);
    setVerificationError(null);

    try {
      // Update API client configuration
      capApiClient.setBaseURL(apiUrl);
      capApiClient.setBearerToken(bearerToken);

      // Transform manifest to Backend API verify context format
      // Use policy_id mode (backend will have default policy loaded)
      const verifyRequest = {
        policy_id: 'lksg.demo.v1', // Hardcoded policy ID (must match backend)
        context: {
          supplier_hashes: [], // Empty array for mock verification (only have root in manifest)
          ubo_hashes: [],      // Empty array for mock verification (only have root in manifest)
          company_commitment_root: manifest.company_commitment_root,
          sanctions_root: undefined,
          jurisdiction_root: undefined,
        },
        backend: 'mock',
        options: {
          check_timestamp: false,
          check_registry: false,
        },
      };

      // Call verify endpoint
      const result = await capApiClient.verifyProofBundle(verifyRequest);

      setVerificationResult(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setVerificationError(errorMessage);
    } finally {
      setIsVerifying(false);
    }
  };

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
                CAP Verifier WebUI
              </h1>
              <p className="text-sm text-gray-500 dark:text-gray-400 mt-1">
                Confidential Assurance Protocol - Proof Verification Interface
              </p>
            </div>

            {manifest && (
              <button
                onClick={handleReset}
                className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 rounded-lg transition-colors"
              >
                Reset
              </button>
            )}
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-6 py-8">
        {/* API Configuration */}
        {!manifest && (
          <div className="mb-8 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-6">
            <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-4">
              API-Konfiguration
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Backend URL
                </label>
                <input
                  type="text"
                  value={apiUrl}
                  onChange={(e) => setApiUrl(e.target.value)}
                  placeholder="http://localhost:8080"
                  className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Bearer Token (optional)
                </label>
                <input
                  type="password"
                  value={bearerToken}
                  onChange={(e) => setBearerToken(e.target.value)}
                  placeholder="eyJ..."
                  className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>
            </div>
          </div>
        )}

        {/* Upload Section */}
        {!manifest && <BundleUploader />}

        {/* Manifest Viewer */}
        {manifest && (
          <div className="space-y-8">
            <ManifestViewer manifest={manifest} />

            {/* Verify Button */}
            <div className="flex justify-center">
              <button
                onClick={handleVerify}
                disabled={!policyHash}
                className="px-8 py-3 text-lg font-semibold text-white bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 disabled:cursor-not-allowed rounded-lg shadow-lg transition-colors"
              >
                Proof Verifizieren
              </button>
            </div>

            {/* Verification Results */}
            <VerificationView />
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="mt-16 bg-white dark:bg-gray-800 border-t border-gray-200 dark:border-gray-700">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <p className="text-center text-sm text-gray-500 dark:text-gray-400">
            CAP v0.11.0 • Built with React + TypeScript + Vite
          </p>
        </div>
      </footer>
    </div>
  );
}

export default App;
