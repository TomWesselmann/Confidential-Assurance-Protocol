/**
 * CAP Bundle Uploader Hook (Server-Side Extraction)
 *
 * @description React hook for uploading proof bundles via API endpoint
 * @architecture Imperative Shell (handles API calls)
 */

import { useState } from 'react';
import { capApiClient } from '../core/api/client';
import type { Manifest } from '../core/models/Manifest';

export interface BundleUploadResult {
  manifest: Manifest;
  policyHash: string;
  proofBundle: string; // Base64-encoded
  packageInfo: {
    size_bytes: number;
    file_count: number;
    files: string[];
  };
}

export interface UseBundleUploaderReturn {
  uploadBundle: (file: File) => Promise<BundleUploadResult>;
  isUploading: boolean;
  error: string | null;
}

/**
 * Hook for uploading CAP proof bundle (.zip) via API
 *
 * @description Uploads ZIP to backend, which extracts:
 * - manifest.json (parsed to Manifest type)
 * - proof.dat (Base64-encoded)
 * - company_commitment_root
 * - package metadata
 */
export function useBundleUploader(): UseBundleUploaderReturn {
  const [isUploading, setIsUploading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const uploadBundle = async (file: File): Promise<BundleUploadResult> => {
    setIsUploading(true);
    setError(null);

    try {
      // Upload to backend API
      const response = await capApiClient.uploadProofPackage(file);

      // Extract policy hash from manifest
      const policyHash = response.manifest.policy.hash;

      return {
        manifest: response.manifest,
        policyHash,
        proofBundle: response.proof_base64,
        packageInfo: response.package_info,
      };
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      throw new Error(`Bundle upload failed: ${errorMessage}`);
    } finally {
      setIsUploading(false);
    }
  };

  return {
    uploadBundle,
    isUploading,
    error,
  };
}
