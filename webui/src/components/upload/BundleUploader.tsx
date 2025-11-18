/**
 * CAP Bundle Uploader Component
 *
 * @description Drag-and-drop file uploader for proof bundles
 * @architecture Imperative Shell (React UI Component)
 */

import { useCallback } from 'react';
import { useDropzone } from 'react-dropzone';
import { useVerificationStore } from '../../store/verificationStore';
import { useBundleUploader } from '../../hooks/useBundleUploader';

export const BundleUploader: React.FC = () => {
  const {
    setUploadedFile,
    setUploadError,
    setManifest,
    setPolicyHash,
    setProofBundle,
  } = useVerificationStore();

  const { uploadBundle, isUploading, error: uploadError } = useBundleUploader();

  const onDrop = useCallback(
    async (acceptedFiles: File[]) => {
      if (acceptedFiles.length === 0) return;

      const file = acceptedFiles[0];
      setUploadedFile(file);
      setUploadError(null);

      try {
        const result = await uploadBundle(file);

        setManifest(result.manifest);
        setPolicyHash(result.policyHash);
        setProofBundle(result.proofBundle);
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';
        setUploadError(errorMessage);
      }
    },
    [uploadBundle, setUploadedFile, setUploadError, setManifest, setPolicyHash, setProofBundle]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    accept: {
      'application/zip': ['.zip'],
    },
    maxFiles: 1,
    disabled: isUploading,
  });

  return (
    <div className="w-full max-w-2xl mx-auto p-6">
      <div
        {...getRootProps()}
        className={`
          border-2 border-dashed rounded-lg p-12 text-center cursor-pointer
          transition-colors duration-200
          ${
            isDragActive
              ? 'border-blue-500 bg-blue-50 dark:bg-blue-900/20'
              : 'border-gray-300 dark:border-gray-600 hover:border-blue-400'
          }
          ${isUploading ? 'opacity-50 cursor-not-allowed' : ''}
        `}
      >
        <input {...getInputProps()} />

        <div className="space-y-4">
          <div className="text-6xl text-gray-400">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              fill="none"
              viewBox="0 0 24 24"
              strokeWidth={1.5}
              stroke="currentColor"
              className="w-16 h-16 mx-auto"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M20.25 7.5l-.625 10.632a2.25 2.25 0 01-2.247 2.118H6.622a2.25 2.25 0 01-2.247-2.118L3.75 7.5M10 11.25h4M3.375 7.5h17.25c.621 0 1.125-.504 1.125-1.125v-1.5c0-.621-.504-1.125-1.125-1.125H3.375c-.621 0-1.125.504-1.125 1.125v1.5c0 .621.504 1.125 1.125 1.125z"
              />
            </svg>
          </div>

          {isUploading ? (
            <div className="space-y-2">
              <p className="text-lg font-semibold text-gray-700 dark:text-gray-300">
                Lade Bundle hoch...
              </p>
              <div className="w-48 h-2 mx-auto bg-gray-200 rounded-full overflow-hidden">
                <div className="h-full bg-blue-500 animate-pulse" style={{ width: '60%' }} />
              </div>
            </div>
          ) : isDragActive ? (
            <p className="text-lg font-semibold text-blue-600 dark:text-blue-400">
              Bundle hier ablegen...
            </p>
          ) : (
            <div className="space-y-2">
              <p className="text-lg font-semibold text-gray-700 dark:text-gray-300">
                CAP Proof Bundle hochladen
              </p>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Ziehen Sie eine .zip-Datei hierher oder klicken Sie zum Auswählen
              </p>
              <p className="text-xs text-gray-400 dark:text-gray-500">
                Das Bundle wird automatisch auf dem Server extrahiert
              </p>
            </div>
          )}
        </div>
      </div>

      {uploadError && (
        <div className="mt-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-sm font-semibold text-red-800 dark:text-red-300">
            Fehler beim Upload
          </p>
          <p className="text-xs text-red-600 dark:text-red-400 mt-1">{uploadError}</p>
        </div>
      )}
    </div>
  );
};
