/**
 * Import View Component
 *
 * @description Step 1 - Import CSV files (Suppliers, UBOs)
 */

import { useState } from 'react';
import { useWorkflowStore } from '../../store/workflowStore';
import {
  selectCsvFile,
  importCsv,
  truncateHash,
  type CsvType,
} from '../../lib/tauri';

interface ImportCardProps {
  title: string;
  csvType: CsvType;
  description: string;
  result: { record_count: number; hash: string } | null;
  isLoading: boolean;
  error: string | null;
  onImport: () => void;
}

const ImportCard: React.FC<ImportCardProps> = ({
  title,
  description,
  result,
  isLoading,
  error,
  onImport,
}) => {
  const isImported = result !== null;

  return (
    <div
      className={`
        border rounded p-3 transition-all duration-200 bg-white dark:bg-gray-800
        ${isImported ? 'border-green-500' : 'border-gray-200 dark:border-gray-700'}
      `}
    >
      <div className="flex items-center justify-between mb-2">
        <div>
          <h3 className="text-xs font-semibold text-gray-900 dark:text-gray-100">{title}</h3>
          <p className="text-[10px] text-gray-500 dark:text-gray-400">{description}</p>
        </div>
        {isImported && (
          <svg className="w-4 h-4 flex-shrink-0 text-green-500" width="16" height="16" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        )}
      </div>

      {isImported && result && (
        <div className="mb-2 p-1.5 bg-green-50 dark:bg-green-900/20 rounded text-[10px]">
          <strong>{result.record_count}</strong> Datensätze
          <span className="ml-1 font-mono text-gray-500">{truncateHash(result.hash, 4)}</span>
        </div>
      )}

      {error && (
        <div className="mb-2 p-1.5 bg-red-50 dark:bg-red-900/20 rounded text-[10px] text-red-600">{error}</div>
      )}

      <button
        onClick={onImport}
        disabled={isLoading}
        className={`w-full py-1 px-2 rounded text-xs font-medium transition-colors
          ${isImported ? 'bg-gray-100 dark:bg-gray-700 text-gray-600' : 'bg-blue-600 text-white hover:bg-blue-700'}
          disabled:opacity-50`}
      >
        {isLoading ? 'Importiere...' : isImported ? 'Erneut' : 'CSV auswählen'}
      </button>
    </div>
  );
};

export const ImportView: React.FC = () => {
  const { projectPath, importResults, setImportResult, goToNextStep, steps } =
    useWorkflowStore();

  const [loadingSuppliers, setLoadingSuppliers] = useState(false);
  const [loadingUbos, setLoadingUbos] = useState(false);
  const [errorSuppliers, setErrorSuppliers] = useState<string | null>(null);
  const [errorUbos, setErrorUbos] = useState<string | null>(null);

  const handleImport = async (csvType: CsvType) => {
    if (!projectPath) return;

    const setLoading = csvType === 'suppliers' ? setLoadingSuppliers : setLoadingUbos;
    const setError = csvType === 'suppliers' ? setErrorSuppliers : setErrorUbos;

    try {
      setLoading(true);
      setError(null);

      // Open file dialog
      const filePath = await selectCsvFile();
      if (!filePath) {
        setLoading(false);
        return;
      }

      // Import CSV
      const result = await importCsv(projectPath, csvType, filePath);

      // Store result
      setImportResult(csvType === 'suppliers' ? 'suppliers' : 'ubos', result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unbekannter Fehler';
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const canProceed = steps.import.status === 'completed';

  return (
    <div className="space-y-3">
      {/* Header - compact */}
      <div className="text-center">
        <h2 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
          Daten importieren
        </h2>
        <p className="text-xs text-gray-500 dark:text-gray-400">
          Lieferanten- und UBO-Daten als CSV
        </p>
      </div>

      {/* Import Cards - compact grid */}
      <div className="grid md:grid-cols-2 gap-3 max-w-2xl mx-auto">
        <ImportCard
          title="Suppliers"
          csvType="suppliers"
          description="name, jurisdiction, tier"
          result={importResults.suppliers}
          isLoading={loadingSuppliers}
          error={errorSuppliers}
          onImport={() => handleImport('suppliers')}
        />

        <ImportCard
          title="UBOs"
          csvType="ubos"
          description="name, birthdate, citizenship"
          result={importResults.ubos}
          isLoading={loadingUbos}
          error={errorUbos}
          onImport={() => handleImport('ubos')}
        />
      </div>

      {/* Navigation - compact */}
      <div className="flex items-center justify-between max-w-2xl mx-auto pt-2">
        <span className="text-[10px] text-gray-500">
          {importResults.suppliers && importResults.ubos ? '2/2' : importResults.suppliers || importResults.ubos ? '1/2' : '0/2'} importiert
        </span>
        <button
          onClick={goToNextStep}
          disabled={!canProceed}
          className={`px-3 py-1.5 rounded text-xs font-medium transition-colors
            ${canProceed ? 'bg-blue-600 text-white hover:bg-blue-700' : 'bg-gray-300 text-gray-500 cursor-not-allowed'}`}
        >
          Weiter
        </button>
      </div>
    </div>
  );
};
