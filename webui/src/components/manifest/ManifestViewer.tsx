/**
 * CAP Manifest Viewer Component
 *
 * @description Displays manifest details in a structured view
 * @architecture Imperative Shell (React UI Component)
 */

import { useState } from 'react';
import type { Manifest } from '../../core/models/Manifest';
import { formatHashShort, formatTimestamp, formatProofType } from '../../core/utils/formatters';
import { isValidHash } from '../../core/utils/validation';

interface ManifestViewerProps {
  manifest: Manifest;
}

export const ManifestViewer: React.FC<ManifestViewerProps> = ({ manifest }) => {
  const [expandedSections, setExpandedSections] = useState<Set<string>>(new Set(['overview']));

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(section)) {
        next.delete(section);
      } else {
        next.add(section);
      }
      return next;
    });
  };

  const renderHash = (hash: string, label: string) => {
    const isValid = isValidHash(hash);

    return (
      <div className="flex items-center justify-between py-2">
        <span className="text-sm font-medium text-gray-700 dark:text-gray-300">{label}:</span>
        <div className="flex items-center space-x-2">
          <code className="text-xs font-mono bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
            {formatHashShort(hash, 12, 10)}
          </code>
          {isValid ? (
            <span className="text-green-500 text-xs">✓</span>
          ) : (
            <span className="text-red-500 text-xs">✗</span>
          )}
        </div>
      </div>
    );
  };

  const Section: React.FC<{ id: string; title: string; children: React.ReactNode }> = ({
    id,
    title,
    children,
  }) => {
    const isExpanded = expandedSections.has(id);

    return (
      <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
        <button
          onClick={() => toggleSection(id)}
          className="w-full bg-gray-50 dark:bg-gray-900 px-6 py-3 flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        >
          <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100">{title}</h3>
          <span className="text-gray-500">{isExpanded ? '−' : '+'}</span>
        </button>

        {isExpanded && <div className="px-6 py-4 bg-white dark:bg-gray-800">{children}</div>}
      </div>
    );
  };

  return (
    <div className="w-full max-w-4xl mx-auto p-6 space-y-4">
      <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-6">
        <h2 className="text-2xl font-bold text-gray-900 dark:text-gray-100 mb-2">
          CAP Manifest
        </h2>
        <p className="text-sm text-gray-500 dark:text-gray-400">
          Version: {manifest.version} • Created: {formatTimestamp(manifest.created_at)}
        </p>
      </div>

      {/* Overview Section */}
      <Section id="overview" title="Overview">
        <div className="space-y-2">
          {renderHash(manifest.supplier_root, 'Supplier Root')}
          {renderHash(manifest.ubo_root, 'UBO Root')}
          {renderHash(manifest.company_commitment_root, 'Company Commitment Root')}
        </div>
      </Section>

      {/* Policy Section */}
      <Section id="policy" title="Policy">
        <div className="space-y-2">
          <div className="flex items-center justify-between py-2">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Name:</span>
            <span className="text-sm text-gray-900 dark:text-gray-100">{manifest.policy.name}</span>
          </div>
          <div className="flex items-center justify-between py-2">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Version:</span>
            <span className="text-sm text-gray-900 dark:text-gray-100">
              {manifest.policy.version}
            </span>
          </div>
          {renderHash(manifest.policy.hash, 'Policy Hash')}
        </div>
      </Section>

      {/* Proof Section */}
      <Section id="proof" title="Proof">
        <div className="space-y-2">
          <div className="flex items-center justify-between py-2">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Type:</span>
            <span className="text-sm text-gray-900 dark:text-gray-100">
              {formatProofType(manifest.proof.type)}
            </span>
          </div>
          <div className="flex items-center justify-between py-2">
            <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Status:</span>
            <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold bg-blue-100 dark:bg-blue-900/30 text-blue-800 dark:text-blue-300">
              {manifest.proof.status}
            </span>
          </div>
        </div>
      </Section>

      {/* Audit Trail Section */}
      <Section id="audit" title="Audit Trail">
        <div className="space-y-4">
          {/* Core Fields */}
          <div className="space-y-2">
            <div className="flex items-center justify-between py-2">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Events Count:
              </span>
              <span className="text-sm font-bold text-gray-900 dark:text-gray-100">
                {manifest.audit.events_count}
              </span>
            </div>
            {renderHash(manifest.audit.tail_digest, 'Tail Digest')}
          </div>

          {/* Extended Fields (if available) */}
          {(manifest.audit.time_range ||
            manifest.audit.event_categories ||
            manifest.audit.last_event_type ||
            manifest.audit.hash_function ||
            manifest.audit.chain_type ||
            manifest.audit.integrity ||
            manifest.audit.audit_chain_version) && (
            <div className="border-t border-gray-200 dark:border-gray-700 pt-4 space-y-2">
              <h4 className="text-sm font-semibold text-gray-900 dark:text-gray-100 mb-3">
                Extended Audit Information
              </h4>

              {manifest.audit.time_range && (
                <div className="flex items-center justify-between py-2">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Time Range:
                  </span>
                  <span className="text-xs text-gray-900 dark:text-gray-100">
                    {formatTimestamp(manifest.audit.time_range.start)} →{' '}
                    {formatTimestamp(manifest.audit.time_range.end)}
                  </span>
                </div>
              )}

              {manifest.audit.event_categories && (
                <div className="flex items-center justify-between py-2">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Event Categories:
                  </span>
                  <span className="text-xs text-gray-900 dark:text-gray-100">
                    {manifest.audit.event_categories.data_changes && (
                      <span>Data: {manifest.audit.event_categories.data_changes}</span>
                    )}
                    {manifest.audit.event_categories.compliance && (
                      <span> • Compliance: {manifest.audit.event_categories.compliance}</span>
                    )}
                    {manifest.audit.event_categories.system && (
                      <span> • System: {manifest.audit.event_categories.system}</span>
                    )}
                  </span>
                </div>
              )}

              {manifest.audit.last_event_type && (
                <div className="flex items-center justify-between py-2">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Last Event:
                  </span>
                  <code className="text-xs font-mono bg-gray-100 dark:bg-gray-800 px-2 py-1 rounded">
                    {manifest.audit.last_event_type}
                  </code>
                </div>
              )}

              {manifest.audit.hash_function && (
                <div className="flex items-center justify-between py-2">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Hash Function:
                  </span>
                  <span className="text-sm text-gray-900 dark:text-gray-100">
                    {manifest.audit.hash_function}
                  </span>
                </div>
              )}

              {manifest.audit.chain_type && (
                <div className="flex items-center justify-between py-2">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Chain Type:
                  </span>
                  <span className="text-sm text-gray-900 dark:text-gray-100">
                    {manifest.audit.chain_type}
                  </span>
                </div>
              )}

              {manifest.audit.integrity && (
                <div className="flex items-center justify-between py-2">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Integrity:
                  </span>
                  <span
                    className={`inline-flex items-center px-3 py-1 rounded-full text-xs font-semibold ${
                      manifest.audit.integrity === 'verified'
                        ? 'bg-green-100 dark:bg-green-900/30 text-green-800 dark:text-green-300'
                        : manifest.audit.integrity === 'failed'
                        ? 'bg-red-100 dark:bg-red-900/30 text-red-800 dark:text-red-300'
                        : 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-800 dark:text-yellow-300'
                    }`}
                  >
                    {manifest.audit.integrity}
                  </span>
                </div>
              )}

              {manifest.audit.audit_chain_version && (
                <div className="flex items-center justify-between py-2">
                  <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                    Audit Chain Version:
                  </span>
                  <span className="text-sm text-gray-900 dark:text-gray-100">
                    {manifest.audit.audit_chain_version}
                  </span>
                </div>
              )}
            </div>
          )}

          {/* Description */}
          <div className="mt-4 p-3 bg-blue-50 dark:bg-blue-900/20 rounded border border-blue-200 dark:border-blue-800">
            <p className="text-xs text-blue-700 dark:text-blue-300">
              The audit trail is an immutable{' '}
              {manifest.audit.hash_function || 'SHA3-256'} hash chain with{' '}
              {manifest.audit.events_count} events
              {manifest.audit.chain_type && ` (${manifest.audit.chain_type})`}
              {manifest.audit.integrity === 'verified' && '. Integrity has been fully verified'}
              .
            </p>
          </div>
        </div>
      </Section>

      {/* Signatures Section */}
      {manifest.signatures.length > 0 && (
        <Section id="signatures" title={`Signatures (${manifest.signatures.length})`}>
          <div className="space-y-4">
            {manifest.signatures.map((sig, index) => (
              <div
                key={index}
                className="p-4 bg-gray-50 dark:bg-gray-900 rounded border border-gray-200 dark:border-gray-700"
              >
                <div className="flex items-center justify-between mb-2">
                  <span className="text-xs font-semibold text-gray-700 dark:text-gray-300">
                    Signature #{index + 1}
                  </span>
                  <span className="text-xs text-gray-500 dark:text-gray-400">
                    {formatTimestamp(sig.signed_at)}
                  </span>
                </div>
                <div className="space-y-1">
                  <div className="flex items-center space-x-2">
                    <span className="text-xs text-gray-600 dark:text-gray-400">KID:</span>
                    <code className="text-xs font-mono text-gray-900 dark:text-gray-100">
                      {sig.kid}
                    </code>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span className="text-xs text-gray-600 dark:text-gray-400">Algorithm:</span>
                    <code className="text-xs font-mono text-gray-900 dark:text-gray-100">
                      {sig.algorithm}
                    </code>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </Section>
      )}

      {/* Time Anchor Section (if present) */}
      {manifest.time_anchor && (
        <Section id="timeanchor" title="Time Anchor (Blockchain)">
          <div className="space-y-2">
            <div className="flex items-center justify-between py-2">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Blockchain:
              </span>
              <span className="text-sm text-gray-900 dark:text-gray-100">
                {manifest.time_anchor.blockchain}
              </span>
            </div>
            <div className="flex items-center justify-between py-2">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Block Number:
              </span>
              <span className="text-sm text-gray-900 dark:text-gray-100">
                {manifest.time_anchor.block_number}
              </span>
            </div>
            <div className="flex items-center justify-between py-2">
              <span className="text-sm font-medium text-gray-700 dark:text-gray-300">
                Timestamp:
              </span>
              <span className="text-sm text-gray-900 dark:text-gray-100">
                {formatTimestamp(manifest.time_anchor.timestamp)}
              </span>
            </div>
          </div>
        </Section>
      )}
    </div>
  );
};
