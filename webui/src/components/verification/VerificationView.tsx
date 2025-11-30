/**
 * CAP Verification View Component
 *
 * @description Displays detailed verification results and bundle information
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
              Verifying proof...
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
            Verification Error
          </h3>
          <p className="text-sm text-red-600 dark:text-red-400">{verificationError}</p>
        </div>
      </div>
    );
  }

  if (!verificationResult) {
    return null;
  }

  // Tauri Backend Response Format (camelCase)
  const { status, bundleId, manifestHash, proofHash, signatureValid, timestampValid, registryMatch, details } = verificationResult;
  const isSuccess = status === 'ok';

  // Truncate hash for display
  const truncateHash = (hash: string | undefined, length = 16) => {
    if (!hash) return 'N/A';
    if (hash.length <= length * 2) return hash;
    return `${hash.slice(0, length)}...${hash.slice(-length)}`;
  };

  return (
    <div className="w-full max-w-4xl mx-auto p-6 space-y-4">
      {/* Status Header */}
      <div
        className={`
        border rounded-lg p-5
        ${
          isSuccess
            ? 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800'
            : 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800'
        }
      `}
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            {/* Status Icon */}
            <div className={`w-12 h-12 rounded-full flex items-center justify-center ${
              isSuccess ? 'bg-green-100 dark:bg-green-900/30' : 'bg-red-100 dark:bg-red-900/30'
            }`}>
              {isSuccess ? (
                <svg className="w-6 h-6 text-green-600 dark:text-green-400" width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              ) : (
                <svg className="w-6 h-6 text-red-600 dark:text-red-400" width="24" height="24" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              )}
            </div>
            <div>
              <h2 className="text-xl font-bold text-gray-900 dark:text-gray-100">
                {isSuccess ? 'Verification Successful' : 'Verification Failed'}
              </h2>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                {details?.checks_passed ?? 0} of {details?.checks_total ?? 0} checks passed
              </p>
            </div>
          </div>

          <div
            className={`px-4 py-2 rounded-full text-sm font-semibold ${
              isSuccess
                ? 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'
                : 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
            }`}
          >
            {status.toUpperCase()}
          </div>
        </div>
      </div>

      {/* Bundle Info Card */}
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
        <div className="bg-gray-50 dark:bg-gray-900 px-4 py-2 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
            Bundle Information
          </h3>
        </div>
        <div className="p-4 grid grid-cols-2 gap-4 text-sm">
          <div>
            <span className="text-gray-500 dark:text-gray-400">Bundle ID:</span>
            <code className="ml-2 text-xs font-mono text-gray-800 dark:text-gray-200">
              {bundleId || 'N/A'}
            </code>
          </div>
          <div>
            <span className="text-gray-500 dark:text-gray-400">Signatures:</span>
            <span className="ml-2 text-gray-800 dark:text-gray-200">
              {details?.signature_count !== undefined ? `${details.signature_count} signature(s)` : 'N/A'}
            </span>
          </div>
        </div>
      </div>

      {/* Cryptographic Verification */}
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
        <div className="bg-gray-50 dark:bg-gray-900 px-4 py-2 border-b border-gray-200 dark:border-gray-700">
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
            Cryptographic Checks
          </h3>
        </div>

        <div className="divide-y divide-gray-200 dark:divide-gray-700">
          {/* Manifest Hash */}
          <div className="px-4 py-3 flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-900 dark:text-gray-100">Manifest Hash</p>
              <code className="text-xs font-mono text-gray-500 dark:text-gray-400 break-all">
                {truncateHash(manifestHash)}
              </code>
            </div>
            <span className="text-green-600 dark:text-green-400">
              <svg className="w-5 h-5" width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </span>
          </div>

          {/* Proof Hash */}
          <div className="px-4 py-3 flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-900 dark:text-gray-100">Proof Hash</p>
              <code className="text-xs font-mono text-gray-500 dark:text-gray-400 break-all">
                {truncateHash(proofHash)}
              </code>
            </div>
            <span className="text-green-600 dark:text-green-400">
              <svg className="w-5 h-5" width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </span>
          </div>

          {/* Signature */}
          <div className="px-4 py-3 flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-900 dark:text-gray-100">Digital Signature</p>
              <p className="text-xs text-gray-500 dark:text-gray-400">
                {signatureValid
                  ? `${details?.signature_count || 1} valid signature(s) present`
                  : 'No valid signature found'}
              </p>
            </div>
            <span className={signatureValid ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'}>
              {signatureValid ? (
                <svg className="w-5 h-5" width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
              ) : (
                <svg className="w-5 h-5" width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              )}
            </span>
          </div>

          {/* Timestamp (optional) */}
          {timestampValid !== undefined && (
            <div className="px-4 py-3 flex items-center justify-between">
              <div>
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">Timestamp</p>
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  {timestampValid ? 'Verified' : 'Not verified'}
                </p>
              </div>
              <span className={timestampValid ? 'text-green-600 dark:text-green-400' : 'text-gray-400'}>
                {timestampValid ? (
                  <svg className="w-5 h-5" width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                  </svg>
                ) : (
                  <svg className="w-5 h-5" width="20" height="20" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                )}
              </span>
            </div>
          )}
        </div>
      </div>

      {/* Statement Validation Details */}
      {details?.statement_validation && details.statement_validation.length > 0 && (
        <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
          <div className="bg-gray-50 dark:bg-gray-900 px-4 py-2 border-b border-gray-200 dark:border-gray-700">
            <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
              Proof Statement Validation
            </h3>
          </div>
          <div className="divide-y divide-gray-100 dark:divide-gray-700">
            {details.statement_validation.map((check: any, idx: number) => (
              <div key={idx} className="px-4 py-2 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <span className={`w-2 h-2 rounded-full ${
                    check.status === 'ok' ? 'bg-green-500' : 'bg-red-500'
                  }`} />
                  <span className="text-sm text-gray-700 dark:text-gray-300 font-medium">
                    {check.field}
                  </span>
                </div>
                <span className={`text-xs font-medium ${
                  check.status === 'ok'
                    ? 'text-green-600 dark:text-green-400'
                    : 'text-red-600 dark:text-red-400'
                }`}>
                  {check.status === 'ok' ? 'Passed' : 'Failed'}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Verification Summary */}
      <div className="bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-700 rounded-lg p-4">
        <h4 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">
          What was verified?
        </h4>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs text-gray-600 dark:text-gray-400">
          <div className="flex items-center gap-2">
            <svg className="w-4 h-4 text-blue-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>Manifest integrity (BLAKE3 hash)</span>
          </div>
          <div className="flex items-center gap-2">
            <svg className="w-4 h-4 text-blue-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>Proof integrity (SHA3-256 hash)</span>
          </div>
          <div className="flex items-center gap-2">
            <svg className="w-4 h-4 text-blue-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>Ed25519 signature validation</span>
          </div>
          <div className="flex items-center gap-2">
            <svg className="w-4 h-4 text-blue-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <span>Statement consistency (commitments)</span>
          </div>
        </div>
      </div>

      {/* Footer Info */}
      <div className="text-center text-xs text-gray-400 dark:text-gray-500">
        Verification performed with Tauri Desktop Backend (Offline)
      </div>
    </div>
  );
};
