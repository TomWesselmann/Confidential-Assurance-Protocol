/**
 * CAP Verification View Component
 *
 * @description Displays verification results and status (Backend API v0.11.0)
 * @architecture Imperative Shell (React UI Component)
 */

import { useVerificationStore } from '../../store/verificationStore';

export const VerificationView: React.FC = () => {
  const { verificationResult, verificationError, isVerifying } = useVerificationStore();

  if (isVerifying) {
    return (
      <div className="w-full max-w-4xl mx-auto p-6">
        <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-8">
          <div className="flex items-center justify-center space-x-4">
            <div className="w-8 h-8 border-4 border-blue-500 border-t-transparent rounded-full animate-spin" />
            <p className="text-lg font-semibold text-blue-700 dark:text-blue-300">
              Verifiziere Proof...
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (verificationError) {
    return (
      <div className="w-full max-w-4xl mx-auto p-6">
        <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-red-800 dark:text-red-300 mb-2">
            Verifikationsfehler
          </h3>
          <p className="text-sm text-red-600 dark:text-red-400">{verificationError}</p>
        </div>
      </div>
    );
  }

  if (!verificationResult) {
    return null;
  }

  // Backend API v0.11.0 Response Format
  const { result, report, manifest_hash, proof_hash } = verificationResult as any;
  const isSuccess = result === 'OK' && report?.status === 'ok';

  return (
    <div className="w-full max-w-4xl mx-auto p-6 space-y-6">
      {/* Status Header */}
      <div
        className={`
        border rounded-lg p-6
        ${
          isSuccess
            ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
            : 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
        }
      `}
      >
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100">
              {isSuccess ? 'Verifikation Erfolgreich' : 'Verifikation Fehlgeschlagen'}
            </h2>
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
              Status: {report?.status || 'unknown'}
            </p>
          </div>

          <div className="text-right">
            <div
              className={`inline-flex items-center px-4 py-2 rounded-full text-sm font-semibold ${
                isSuccess
                  ? 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'
                  : 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
              }`}
            >
              {result}
            </div>
          </div>
        </div>
      </div>

      {/* Verification Details */}
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
        <div className="bg-gray-50 dark:bg-gray-900 px-6 py-3 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
            Verifikations-Details
          </h3>
        </div>

        <div className="divide-y divide-gray-200 dark:divide-gray-700">
          {/* Manifest Hash */}
          <div className="px-6 py-4 flex items-center justify-between">
            <div className="flex-1">
              <p className="text-sm font-medium text-gray-900 dark:text-gray-100">Manifest Hash</p>
              <code className="text-xs font-mono text-gray-600 dark:text-gray-400 mt-1 block">
                {manifest_hash || report?.manifest_hash || 'N/A'}
              </code>
            </div>
          </div>

          {/* Proof Hash */}
          <div className="px-6 py-4 flex items-center justify-between">
            <div className="flex-1">
              <p className="text-sm font-medium text-gray-900 dark:text-gray-100">Proof Hash</p>
              <code className="text-xs font-mono text-gray-600 dark:text-gray-400 mt-1 block">
                {proof_hash || report?.proof_hash || 'N/A'}
              </code>
            </div>
          </div>

          {/* Signature Valid */}
          {report?.signature_valid !== undefined && (
            <div className="px-6 py-4 flex items-center justify-between">
              <div className="flex-1">
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  Signatur Gültig
                </p>
              </div>
              <div>
                <span
                  className={`inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold ${
                    report.signature_valid
                      ? 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'
                      : 'bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-300'
                  }`}
                >
                  {report.signature_valid ? 'Ja' : 'Nein'}
                </span>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Additional Info */}
      <div className="bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg p-4">
        <p className="text-xs text-blue-700 dark:text-blue-300">
          Verifikation durchgeführt mit Backend API v0.11.0 (Mock Mode)
        </p>
      </div>
    </div>
  );
};
