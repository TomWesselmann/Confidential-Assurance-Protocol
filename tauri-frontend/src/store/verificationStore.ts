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
  isVerifying: boolean;

  // Actions
  setVerificationResult: (result: VerifyBundleResponse | null) => void;
  setVerificationError: (error: string | null) => void;
  setIsVerifying: (isVerifying: boolean) => void;

  reset: () => void;
}

const initialState = {
  verificationResult: null,
  verificationError: null,
  isVerifying: false,
};

export const useVerificationStore = create<VerificationState>((set) => ({
  ...initialState,

  setVerificationResult: (result) =>
    set({ verificationResult: result, verificationError: null, isVerifying: false }),

  setVerificationError: (error) =>
    set({ verificationError: error, verificationResult: null, isVerifying: false }),

  setIsVerifying: (isVerifying) =>
    set({ isVerifying }),

  reset: () => set(initialState),
}));
