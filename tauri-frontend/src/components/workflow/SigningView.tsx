/**
 * Signing View Component
 *
 * @description Optional step after Manifest - Sign the manifest with Ed25519
 * @architecture Thin UI - all crypto operations in Tauri backend
 */

import { useState, useEffect, useCallback } from 'react';
import { useWorkflowStore } from '../../store/workflowStore';
import {
  listKeys,
  generateKeys,
  signProjectManifest,
  verifyManifestSignature,
  validateSignerName,
  truncateHash,
  type KeyInfo,
  type SignResult,
  type SignatureVerifyResult,
} from '../../lib/tauri';

export const SigningView: React.FC = () => {
  const { projectPath, manifestResult } = useWorkflowStore();

  // State
  const [keys, setKeys] = useState<KeyInfo[]>([]);
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [signResult, setSignResult] = useState<SignResult | null>(null);
  const [verifyResult, setVerifyResult] = useState<SignatureVerifyResult | null>(null);

  // UI state
  const [showKeyForm, setShowKeyForm] = useState(false);
  const [newSignerName, setNewSignerName] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isSigning, setIsSigning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationError, setValidationError] = useState<string | null>(null);

  const loadKeys = useCallback(async () => {
    if (!projectPath) return;
    try {
      setIsLoading(true);
      const keyList = await listKeys(projectPath);
      setKeys(keyList);
      if (keyList.length > 0 && !selectedKey) {
        setSelectedKey(keyList[0].signerName);
      }
    } catch (err) {
      console.error('Failed to load keys:', err);
    } finally {
      setIsLoading(false);
    }
  }, [projectPath, selectedKey]);

  const checkExistingSignature = useCallback(async () => {
    if (!projectPath) return;
    try {
      const result = await verifyManifestSignature(projectPath);
      if (!result.error?.includes('no signatures')) {
        setVerifyResult(result);
      }
    } catch {
      // No signature yet - that's fine
    }
  }, [projectPath]);

  // Load keys on mount
  useEffect(() => {
    if (projectPath) {
      loadKeys();
      checkExistingSignature();
    }
  }, [projectPath, loadKeys, checkExistingSignature]);

  const handleGenerateKey = async () => {
    if (!projectPath) return;

    // Client-side validation
    const validation = validateSignerName(newSignerName);
    if (validation !== true) {
      setValidationError(validation);
      return;
    }
    setValidationError(null);

    try {
      setIsGenerating(true);
      setError(null);
      await generateKeys(projectPath, newSignerName);
      setNewSignerName('');
      setShowKeyForm(false);
      await loadKeys();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate key');
    } finally {
      setIsGenerating(false);
    }
  };

  const handleSign = async () => {
    if (!projectPath || !selectedKey) return;

    try {
      setIsSigning(true);
      setError(null);
      const result = await signProjectManifest(projectPath, selectedKey);
      setSignResult(result);
      // Verify after signing
      const verify = await verifyManifestSignature(projectPath);
      setVerifyResult(verify);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Signing failed');
    } finally {
      setIsSigning(false);
    }
  };

  const handleVerify = async () => {
    if (!projectPath) return;
    try {
      setIsLoading(true);
      const result = await verifyManifestSignature(projectPath);
      setVerifyResult(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Verification failed');
    } finally {
      setIsLoading(false);
    }
  };

  // Don't render if no manifest
  if (!manifestResult) {
    return null;
  }

  const isSigned = verifyResult && verifyResult.valid;

  return (
    <div className="mt-4 border-t border-gray-200 dark:border-gray-700 pt-4">
      <div className="text-center mb-3">
        <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
          Sign Manifest (optional)
        </h3>
        <p className="text-xs text-gray-500">Ed25519 signature for authenticity</p>
      </div>

      {/* Error display */}
      {error && (
        <div className="mb-3 p-2 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-xs text-red-600 dark:text-red-400">
          {error}
        </div>
      )}

      {/* Already signed state */}
      {isSigned && (
        <div className="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded p-3 mb-3">
          <div className="flex items-center gap-2 mb-2">
            <svg className="w-4 h-4 text-green-600 dark:text-green-400" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
            <span className="text-sm font-medium text-green-700 dark:text-green-300">Manifest signed</span>
          </div>
          <div className="text-xs text-gray-600 dark:text-gray-400 space-y-1">
            <div className="flex justify-between">
              <span>Signer:</span>
              <span className="font-medium">{verifyResult.signer}</span>
            </div>
            <div className="flex justify-between">
              <span>Algorithm:</span>
              <span className="font-mono">{verifyResult.algorithm}</span>
            </div>
            {signResult && (
              <div className="flex justify-between">
                <span>Signature:</span>
                <code className="font-mono text-blue-600 dark:text-blue-400">
                  {truncateHash(signResult.signatureHash, 8)}
                </code>
              </div>
            )}
          </div>
          <button
            onClick={handleVerify}
            disabled={isLoading}
            className="mt-2 w-full text-xs text-green-600 dark:text-green-400 hover:underline"
          >
            Re-verify signature
          </button>
        </div>
      )}

      {/* Not signed state */}
      {!isSigned && (
        <>
          {/* Key selection or generation */}
          {keys.length === 0 && !showKeyForm ? (
            <div className="text-center py-4">
              <svg className="w-8 h-8 mx-auto text-gray-400 mb-2" width="32" height="32" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
              </svg>
              <p className="text-xs text-gray-500 mb-2">No keys available yet</p>
              <button
                onClick={() => setShowKeyForm(true)}
                className="px-3 py-1.5 text-xs font-medium text-blue-600 bg-blue-50 dark:bg-blue-900/30 dark:text-blue-400 rounded hover:bg-blue-100 dark:hover:bg-blue-900/50"
              >
                Generate Key
              </button>
            </div>
          ) : showKeyForm ? (
            /* Key generation form */
            <div className="bg-gray-50 dark:bg-gray-800 rounded p-3 mb-3">
              <div className="mb-2">
                <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Signer Name
                </label>
                <input
                  type="text"
                  value={newSignerName}
                  onChange={(e) => {
                    setNewSignerName(e.target.value);
                    setValidationError(null);
                  }}
                  placeholder="e.g. Company Name"
                  className="w-full px-2 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                  maxLength={64}
                />
                {validationError && (
                  <p className="mt-1 text-xs text-red-500">{validationError}</p>
                )}
                <p className="mt-1 text-xs text-gray-400">
                  Allowed: letters, numbers, spaces, hyphen, underscore
                </p>
              </div>
              <div className="flex gap-2">
                <button
                  onClick={handleGenerateKey}
                  disabled={isGenerating || !newSignerName.trim()}
                  className="flex-1 px-3 py-1.5 text-xs font-medium text-white bg-blue-600 rounded hover:bg-blue-700 disabled:opacity-50"
                >
                  {isGenerating ? 'Generating...' : 'Generate Key'}
                </button>
                <button
                  onClick={() => {
                    setShowKeyForm(false);
                    setNewSignerName('');
                    setValidationError(null);
                  }}
                  className="px-3 py-1.5 text-xs font-medium text-gray-600 dark:text-gray-400 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600"
                >
                  Cancel
                </button>
              </div>
            </div>
          ) : (
            /* Key selection and signing */
            <div className="space-y-3">
              {/* Key selector */}
              <div>
                <label className="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Sign with key
                </label>
                <select
                  value={selectedKey || ''}
                  onChange={(e) => setSelectedKey(e.target.value)}
                  className="w-full px-2 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                >
                  {keys.map((key) => (
                    <option key={key.kid} value={key.signerName}>
                      {key.signerName} ({key.fingerprint})
                    </option>
                  ))}
                </select>
              </div>

              {/* Selected key details */}
              {selectedKey && keys.length > 0 && (
                <div className="text-xs text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800 rounded p-2">
                  {(() => {
                    const key = keys.find((k) => k.signerName === selectedKey);
                    if (!key) return null;
                    return (
                      <div className="space-y-0.5">
                        <div className="flex justify-between">
                          <span>KID:</span>
                          <code className="font-mono">{truncateHash(key.kid, 8)}</code>
                        </div>
                        <div className="flex justify-between">
                          <span>Fingerprint:</span>
                          <code className="font-mono">{key.fingerprint}</code>
                        </div>
                      </div>
                    );
                  })()}
                </div>
              )}

              {/* Sign button */}
              <button
                onClick={handleSign}
                disabled={isSigning || !selectedKey}
                className="w-full px-3 py-2 text-sm font-medium text-white bg-blue-600 rounded hover:bg-blue-700 disabled:opacity-50 flex items-center justify-center gap-2"
              >
                {isSigning ? (
                  <>
                    <svg className="animate-spin w-4 h-4" width="16" height="16" fill="none" viewBox="0 0 24 24">
                      <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                      <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                    </svg>
                    Signing...
                  </>
                ) : (
                  <>
                    <svg className="w-4 h-4" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
                    </svg>
                    Sign Manifest
                  </>
                )}
              </button>

              {/* Add new key option */}
              <button
                onClick={() => setShowKeyForm(true)}
                className="w-full text-xs text-gray-500 hover:text-gray-700 dark:hover:text-gray-300"
              >
                + Generate new key
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
};
