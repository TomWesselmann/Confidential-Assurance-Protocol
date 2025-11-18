/**
 * CAP Verification Store
 *
 * @description Zustand state management for verification workflow
 * @architecture Imperative Shell (manages UI state)
 */

import { create } from 'zustand';
import type { Manifest } from '../core/models/Manifest';
import type { VerificationResult } from '../core/models/VerificationResult';

export interface VerificationState {
  // Upload state
  uploadedFile: File | null;
  uploadProgress: number;
  uploadError: string | null;

  // Extracted data from bundle
  manifest: Manifest | null;
  policyHash: string | null;
  proofBundle: string | null; // Base64-encoded

  // Verification state
  isVerifying: boolean;
  verificationResult: VerificationResult | null;
  verificationError: string | null;

  // Actions
  setUploadedFile: (file: File | null) => void;
  setUploadProgress: (progress: number) => void;
  setUploadError: (error: string | null) => void;

  setManifest: (manifest: Manifest | null) => void;
  setPolicyHash: (hash: string | null) => void;
  setProofBundle: (bundle: string | null) => void;

  setIsVerifying: (isVerifying: boolean) => void;
  setVerificationResult: (result: VerificationResult | null) => void;
  setVerificationError: (error: string | null) => void;

  reset: () => void;
}

const initialState = {
  uploadedFile: null,
  uploadProgress: 0,
  uploadError: null,
  manifest: null,
  policyHash: null,
  proofBundle: null,
  isVerifying: false,
  verificationResult: null,
  verificationError: null,
};

export const useVerificationStore = create<VerificationState>((set) => ({
  ...initialState,

  setUploadedFile: (file) => set({ uploadedFile: file }),
  setUploadProgress: (progress) => set({ uploadProgress: progress }),
  setUploadError: (error) => set({ uploadError: error }),

  setManifest: (manifest) => set({ manifest }),
  setPolicyHash: (hash) => set({ policyHash: hash }),
  setProofBundle: (bundle) => set({ proofBundle: bundle }),

  setIsVerifying: (isVerifying) => set({ isVerifying }),
  setVerificationResult: (result) => set({ verificationResult: result }),
  setVerificationError: (error) => set({ verificationError: error }),

  reset: () => set(initialState),
}));
