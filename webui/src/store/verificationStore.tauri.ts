/**
 * CAP Verification Store (Tauri Version)
 *
 * @description Zustand state management for offline verification workflow
 * @architecture Imperative Shell (manages UI state)
 * @offline Simplified for Tauri - no upload state needed
 */

import { create } from 'zustand';
import type { VerifyBundleResponse } from '../lib/tauri';

export interface VerificationState {
  // Verification state
  verificationResult: VerifyBundleResponse | null;
  verificationError: string | null;

  // Actions
  setVerificationResult: (result: VerifyBundleResponse | null) => void;
  setVerificationError: (error: string | null) => void;

  reset: () => void;
}

const initialState = {
  verificationResult: null,
  verificationError: null,
};

export const useVerificationStore = create<VerificationState>((set) => ({
  ...initialState,

  setVerificationResult: (result) =>
    set({ verificationResult: result, verificationError: null }),

  setVerificationError: (error) =>
    set({ verificationError: error, verificationResult: null }),

  reset: () => set(initialState),
}));
