/**
 * CAP Bundle Extractor Hook
 *
 * @description React hook for extracting manifest.json and proof.zip from uploaded bundle
 * @architecture Imperative Shell (handles File I/O)
 */

import { useState } from 'react';
import JSZip from 'jszip';
import type { Manifest } from '../core/models/Manifest';
import { isValidManifest } from '../core/models/Manifest';

export interface BundleExtractionResult {
  manifest: Manifest;
  policyHash: string;
  proofBundle: string; // Base64-encoded
}

export interface UseBundleExtractorReturn {
  extractBundle: (file: File) => Promise<BundleExtractionResult>;
  isExtracting: boolean;
  error: string | null;
}

/**
 * Hook for extracting CAP proof bundle (.zip)
 *
 * @description Extracts:
 * - manifest.json (parsed to Manifest type)
 * - proof.zip (Base64-encoded for API)
 * - policy_hash from manifest
 */
export function useBundleExtractor(): UseBundleExtractorReturn {
  const [isExtracting, setIsExtracting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const extractBundle = async (file: File): Promise<BundleExtractionResult> => {
    setIsExtracting(true);
    setError(null);

    try {
      // Handle direct manifest.json upload (for testing)
      if (file.name.endsWith('.json')) {
        const text = await file.text();
        const data = JSON.parse(text);

        if (!isValidManifest(data)) {
          throw new Error('Invalid manifest structure');
        }

        const manifest = data as Manifest;
        const policyHash = manifest.policy.hash;

        return {
          manifest,
          policyHash,
          proofBundle: '', // Empty for direct manifest upload
        };
      }

      // Handle .zip bundle extraction
      if (file.name.endsWith('.zip')) {
        const zip = new JSZip();
        const zipContent = await zip.loadAsync(file);

        // Extract manifest.json
        const manifestFile = zipContent.file('manifest.json');
        if (!manifestFile) {
          throw new Error('manifest.json not found in bundle');
        }

        const manifestText = await manifestFile.async('text');
        const manifestData = JSON.parse(manifestText);

        if (!isValidManifest(manifestData)) {
          throw new Error('Invalid manifest structure');
        }

        const manifest = manifestData as Manifest;
        const policyHash = manifest.policy.hash;

        // Extract proof file (proof.zip or proof.dat) and encode to Base64
        let proofFile = zipContent.file('proof.zip');
        if (!proofFile) {
          proofFile = zipContent.file('proof.dat');
        }

        let proofBundle = '';

        if (proofFile) {
          const proofBlob = await proofFile.async('blob');
          const arrayBuffer = await proofBlob.arrayBuffer();
          const uint8Array = new Uint8Array(arrayBuffer);
          proofBundle = btoa(String.fromCharCode(...uint8Array));
        }

        return {
          manifest,
          policyHash,
          proofBundle,
        };
      }

      throw new Error('Unsupported file type. Please upload .zip or .json');
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      throw new Error(`Bundle extraction failed: ${errorMessage}`);
    } finally {
      setIsExtracting(false);
    }
  };

  return {
    extractBundle,
    isExtracting,
    error,
  };
}
