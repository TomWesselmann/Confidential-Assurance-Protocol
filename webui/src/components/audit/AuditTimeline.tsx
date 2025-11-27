/**
 * AuditTimeline Component
 *
 * Displays audit events in a timeline view with hash chain verification status.
 */

import { useState, useEffect } from 'react';
import {
  getAuditLog,
  verifyAuditChain,
  formatAuditTimestamp,
  getEventTypeName,
  truncateHash,
  type AuditLog,
  type AuditEvent,
  type ChainVerifyResult,
} from '../../lib/tauri';

interface AuditTimelineProps {
  projectPath: string;
}

export function AuditTimeline({ projectPath }: AuditTimelineProps) {
  const [auditLog, setAuditLog] = useState<AuditLog | null>(null);
  const [chainResult, setChainResult] = useState<ChainVerifyResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedEvent, setSelectedEvent] = useState<AuditEvent | null>(null);

  useEffect(() => {
    loadAuditData();
  }, [projectPath]);

  async function loadAuditData() {
    if (!projectPath) return;

    setLoading(true);
    setError(null);

    try {
      const [log, verify] = await Promise.all([
        getAuditLog(projectPath, 100, 0),
        verifyAuditChain(projectPath),
      ]);
      setAuditLog(log);
      setChainResult(verify);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-2 text-gray-600">Lade Audit-Log...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-lg">
        <p className="text-red-600 text-sm">{error}</p>
        <button
          onClick={loadAuditData}
          className="mt-2 text-sm text-red-600 hover:text-red-800 underline"
        >
          Erneut versuchen
        </button>
      </div>
    );
  }

  if (!auditLog || auditLog.events.length === 0) {
    return (
      <div className="p-8 text-center">
        <svg
          className="w-12 h-12 mx-auto text-gray-300 mb-4"
          width="48"
          height="48"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
          />
        </svg>
        <p className="text-gray-500">Keine Audit-Events vorhanden</p>
        <p className="text-gray-400 text-sm mt-1">
          Events werden bei Workflow-Aktionen automatisch aufgezeichnet
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Chain Status Banner */}
      <div
        className={`p-3 rounded-lg flex items-center justify-between ${
          chainResult?.valid
            ? 'bg-green-50 border border-green-200'
            : 'bg-red-50 border border-red-200'
        }`}
      >
        <div className="flex items-center gap-2">
          {chainResult?.valid ? (
            <svg className="w-5 h-5 text-green-600" width="20" height="20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
          ) : (
            <svg className="w-5 h-5 text-red-600" width="20" height="20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          )}
          <span className={chainResult?.valid ? 'text-green-700' : 'text-red-700'}>
            {chainResult?.valid
              ? `Hash-Chain konsistent (${chainResult.verifiedCount} Events)`
              : `Hash-Chain inkonsistent (${chainResult?.errors.length} Fehler)`}
          </span>
        </div>
        {chainResult?.tailHash && (
          <span className="text-xs font-mono text-gray-500">
            Tail: {truncateHash(chainResult.tailHash, 6)}
          </span>
        )}
      </div>

      {/* Timeline */}
      <div className="relative">
        {/* Vertical line */}
        <div className="absolute left-4 top-0 bottom-0 w-0.5 bg-gray-200"></div>

        {/* Events */}
        <div className="space-y-4">
          {auditLog.events.map((event, index) => (
            <div key={index} className="relative pl-10">
              {/* Timeline dot */}
              <div
                className={`absolute left-2.5 w-3 h-3 rounded-full border-2 ${
                  getEventColor(event)
                }`}
              ></div>

              {/* Event card */}
              <div
                className={`p-3 rounded-lg border cursor-pointer transition-colors ${
                  selectedEvent === event
                    ? 'bg-blue-50 border-blue-300'
                    : 'bg-white border-gray-200 hover:bg-gray-50'
                }`}
                onClick={() => setSelectedEvent(selectedEvent === event ? null : event)}
              >
                <div className="flex items-start justify-between">
                  <div>
                    <span className="font-medium text-gray-900">
                      {getEventTypeName(event.event)}
                    </span>
                    <span className="ml-2 text-xs text-gray-500">
                      {event.seq !== undefined && `#${event.seq}`}
                    </span>
                  </div>
                  <span className="text-xs text-gray-400">
                    {formatAuditTimestamp(event.ts)}
                  </span>
                </div>

                {/* Expanded details */}
                {selectedEvent === event && (
                  <div className="mt-3 pt-3 border-t border-gray-100">
                    <EventDetails event={event} />
                  </div>
                )}
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Pagination info */}
      {auditLog.totalCount > auditLog.events.length && (
        <div className="text-center text-sm text-gray-500">
          Zeige {auditLog.events.length} von {auditLog.totalCount} Events
        </div>
      )}
    </div>
  );
}

function EventDetails({ event }: { event: AuditEvent }) {
  return (
    <div className="space-y-2 text-xs">
      {/* Hashes */}
      <div className="grid grid-cols-2 gap-2">
        <div>
          <span className="text-gray-500">Self Hash:</span>
          <div className="font-mono text-gray-700 break-all">{event.selfHash}</div>
        </div>
        <div>
          <span className="text-gray-500">Prev Hash:</span>
          <div className="font-mono text-gray-700 break-all">{event.prevHash}</div>
        </div>
      </div>

      {/* V2.0 specific fields */}
      {event.policyId && (
        <div>
          <span className="text-gray-500">Policy ID:</span>
          <span className="ml-2 text-gray-700">{event.policyId}</span>
        </div>
      )}
      {event.manifestHash && (
        <div>
          <span className="text-gray-500">Manifest Hash:</span>
          <span className="ml-2 font-mono text-gray-700">{truncateHash(event.manifestHash, 8)}</span>
        </div>
      )}
      {event.result && (
        <div>
          <span className="text-gray-500">Result:</span>
          <span
            className={`ml-2 px-1.5 py-0.5 rounded text-xs ${
              event.result === 'OK'
                ? 'bg-green-100 text-green-700'
                : event.result === 'WARN'
                ? 'bg-yellow-100 text-yellow-700'
                : 'bg-red-100 text-red-700'
            }`}
          >
            {event.result}
          </span>
        </div>
      )}
      {event.runId && (
        <div>
          <span className="text-gray-500">Run ID:</span>
          <span className="ml-2 font-mono text-gray-700">{event.runId}</span>
        </div>
      )}

      {/* V1.0 details */}
      {event.details && Object.keys(event.details).length > 0 && (
        <div>
          <span className="text-gray-500">Details:</span>
          <pre className="mt-1 p-2 bg-gray-50 rounded text-xs overflow-auto max-h-32">
            {JSON.stringify(event.details, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
}

function getEventColor(event: AuditEvent): string {
  if (event.result === 'FAIL') {
    return 'bg-red-500 border-red-500';
  }
  if (event.result === 'WARN') {
    return 'bg-yellow-500 border-yellow-500';
  }

  // Color by event type
  const colors: Record<string, string> = {
    project_created: 'bg-blue-500 border-blue-500',
    csv_imported: 'bg-cyan-500 border-cyan-500',
    commitments_created: 'bg-purple-500 border-purple-500',
    policy_loaded: 'bg-indigo-500 border-indigo-500',
    manifest_built: 'bg-violet-500 border-violet-500',
    proof_built: 'bg-green-500 border-green-500',
    bundle_exported: 'bg-emerald-500 border-emerald-500',
    bundle_verifier_run: 'bg-teal-500 border-teal-500',
  };

  return colors[event.event] || 'bg-gray-500 border-gray-500';
}

export default AuditTimeline;
