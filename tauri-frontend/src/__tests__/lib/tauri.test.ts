/**
 * CAP Tauri API Tests
 *
 * @description Comprehensive unit tests for Tauri API wrapper functions
 * Tests pure utility functions, mocked API calls, error handling
 */

import { describe, it, expect, vi, beforeEach, afterEach, type Mock } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';

// Mock Tauri core
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

// Mock Tauri event
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

// Mock Tauri dialog plugin
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}));

// Import functions after mocking
import {
  truncateHash,
  formatFileSize,
  formatAuditTimestamp,
  getEventTypeName,
  validateSignerName,
  verifyBundle,
  getBundleInfo,
  createProject,
  listProjects,
  listAllProjects,
  createNewProject,
  createTempProject,
  createProjectInFolder,
  getProjectStatus,
  importCsv,
  createCommitments,
  loadPolicy,
  buildManifest,
  buildProof,
  exportBundle,
  getAuditLog,
  verifyAuditChain,
  generateKeys,
  listKeys,
  signProjectManifest,
  verifyManifestSignature,
  selectBundleFile,
  selectBundleDirectory,
  selectCsvFile,
  selectPolicyFile,
  selectExportPath,
  selectWorkspace,
  selectWorkingFolder,
  selectWorkspaceFolder,
  getAppInfo,
  setWorkspacePath,
  resetWorkspacePath,
  readFileContent,
  readPolicyContent,
  readManifestContent,
} from '../../lib/tauri';

describe('tauri utilities', () => {
  describe('truncateHash', () => {
    it('should truncate long hash', () => {
      const hash = '0x' + 'a'.repeat(64);
      const result = truncateHash(hash);

      expect(result).toBe('0xaaaaaaaa...aaaaaaaa');
    });

    it('should use custom character count', () => {
      const hash = '0x' + 'a'.repeat(64);
      const result = truncateHash(hash, 4);

      expect(result).toBe('0xaaaa...aaaa');
    });

    it('should return original if shorter than truncation', () => {
      const hash = '0x1234';
      const result = truncateHash(hash, 8);

      expect(result).toBe('0x1234');
    });

    it('should handle empty string', () => {
      const result = truncateHash('');
      expect(result).toBe('');
    });

    it('should handle null-ish values', () => {
      const result = truncateHash(null as unknown as string);
      expect(result).toBeFalsy();
    });

    it('should handle exact boundary length', () => {
      // 8 chars * 2 + 3 (for ...) = 19
      const hash = '0x' + 'a'.repeat(17); // total 19
      const result = truncateHash(hash, 8);
      expect(result).toBe('0x' + 'a'.repeat(17)); // no truncation
    });
  });

  describe('formatFileSize', () => {
    it('should format 0 bytes', () => {
      expect(formatFileSize(0)).toBe('0 B');
    });

    it('should format bytes', () => {
      expect(formatFileSize(512)).toBe('512 B');
    });

    it('should format kilobytes', () => {
      expect(formatFileSize(1024)).toBe('1 KB');
      expect(formatFileSize(1536)).toBe('1.5 KB');
    });

    it('should format megabytes', () => {
      expect(formatFileSize(1048576)).toBe('1 MB');
    });

    it('should format gigabytes', () => {
      expect(formatFileSize(1073741824)).toBe('1 GB');
    });

    it('should show one decimal place', () => {
      const result = formatFileSize(1500);
      expect(result).toBe('1.5 KB');
    });

    it('should handle large values', () => {
      expect(formatFileSize(2147483648)).toBe('2 GB');
    });
  });

  describe('formatAuditTimestamp', () => {
    it('should format ISO timestamp to German locale', () => {
      const timestamp = '2025-12-14T10:30:45.000Z';
      const result = formatAuditTimestamp(timestamp);

      // Should contain date in DD.MM.YYYY format
      expect(result).toMatch(/\d{2}\.\d{2}\.\d{4}/);
      // Should contain time
      expect(result).toMatch(/\d{2}:\d{2}:\d{2}/);
    });

    it('should return Invalid Date on invalid timestamp', () => {
      const invalid = 'not-a-date';
      const result = formatAuditTimestamp(invalid);

      // The function returns 'Invalid Date' for unparseable timestamps
      expect(result).toBe('Invalid Date');
    });

    it('should handle different dates', () => {
      const result1 = formatAuditTimestamp('2025-01-01T00:00:00.000Z');
      const result2 = formatAuditTimestamp('2025-12-31T23:59:59.000Z');

      expect(result1).not.toBe(result2);
    });

    it('should handle timestamp without milliseconds', () => {
      const timestamp = '2025-06-15T14:30:00Z';
      const result = formatAuditTimestamp(timestamp);
      expect(result).toMatch(/\d{2}\.\d{2}\.\d{4}/);
    });
  });

  describe('getEventTypeName', () => {
    it('should return German name for project_created', () => {
      expect(getEventTypeName('project_created')).toBe('Projekt erstellt');
    });

    it('should return German name for csv_imported', () => {
      expect(getEventTypeName('csv_imported')).toBe('CSV importiert');
    });

    it('should return German name for commitments_created', () => {
      expect(getEventTypeName('commitments_created')).toBe('Commitments erstellt');
    });

    it('should return German name for policy_loaded', () => {
      expect(getEventTypeName('policy_loaded')).toBe('Policy geladen');
    });

    it('should return German name for manifest_built', () => {
      expect(getEventTypeName('manifest_built')).toBe('Manifest erstellt');
    });

    it('should return German name for proof_built', () => {
      expect(getEventTypeName('proof_built')).toBe('Proof erstellt');
    });

    it('should return German name for bundle_exported', () => {
      expect(getEventTypeName('bundle_exported')).toBe('Bundle exportiert');
    });

    it('should return German name for bundle_verifier_run', () => {
      expect(getEventTypeName('bundle_verifier_run')).toBe('Bundle verifiziert');
    });

    it('should return German name for keys_generated', () => {
      expect(getEventTypeName('keys_generated')).toBe('Schlüssel generiert');
    });

    it('should return German name for manifest_signed', () => {
      expect(getEventTypeName('manifest_signed')).toBe('Manifest signiert');
    });

    it('should return German name for signature_verified', () => {
      expect(getEventTypeName('signature_verified')).toBe('Signatur verifiziert');
    });

    it('should return German name for registry_entry_added', () => {
      expect(getEventTypeName('registry_entry_added')).toBe('Registry-Eintrag hinzugefügt');
    });

    it('should return German name for verify_response', () => {
      expect(getEventTypeName('verify_response')).toBe('Verifikationsantwort');
    });

    it('should return German name for policy_compile', () => {
      expect(getEventTypeName('policy_compile')).toBe('Policy kompiliert');
    });

    it('should return original for unknown event types', () => {
      expect(getEventTypeName('unknown_event')).toBe('unknown_event');
      expect(getEventTypeName('custom_action')).toBe('custom_action');
    });
  });

  describe('validateSignerName', () => {
    it('should accept valid names', () => {
      expect(validateSignerName('Test Company')).toBe(true);
      expect(validateSignerName('Company-Name')).toBe(true);
      expect(validateSignerName('Company_Name')).toBe(true);
      expect(validateSignerName('Company123')).toBe(true);
    });

    it('should reject empty name', () => {
      const result = validateSignerName('');
      expect(result).toBe('Signer-Name ist erforderlich');
    });

    it('should reject whitespace-only name', () => {
      const result = validateSignerName('   ');
      expect(result).toBe('Signer-Name ist erforderlich');
    });

    it('should reject name longer than 64 characters', () => {
      const longName = 'a'.repeat(65);
      const result = validateSignerName(longName);
      expect(result).toBe('Signer-Name zu lang (max. 64 Zeichen)');
    });

    it('should accept name with exactly 64 characters', () => {
      const name = 'a'.repeat(64);
      expect(validateSignerName(name)).toBe(true);
    });

    it('should reject name with path traversal characters (..)', () => {
      const result = validateSignerName('test..company');
      expect(result).toBe('Ungültiger Signer-Name: Pfad-Zeichen nicht erlaubt');
    });

    it('should reject name with forward slash', () => {
      const result = validateSignerName('test/company');
      expect(result).toBe('Ungültiger Signer-Name: Pfad-Zeichen nicht erlaubt');
    });

    it('should reject name with backslash', () => {
      const result = validateSignerName('test\\company');
      expect(result).toBe('Ungültiger Signer-Name: Pfad-Zeichen nicht erlaubt');
    });

    it('should reject name with special characters', () => {
      expect(validateSignerName('test@company')).toContain('Ungültiger Signer-Name');
      expect(validateSignerName('test#company')).toContain('Ungültiger Signer-Name');
      expect(validateSignerName('test$company')).toContain('Ungültiger Signer-Name');
      expect(validateSignerName('test%company')).toContain('Ungültiger Signer-Name');
    });

    it('should reject name with unicode characters', () => {
      expect(validateSignerName('Müller GmbH')).toContain('Ungültiger Signer-Name');
      expect(validateSignerName('Company äöü')).toContain('Ungültiger Signer-Name');
    });
  });
});

describe('tauri API calls', () => {
  const mockInvoke = invoke as Mock;
  const mockListen = listen as Mock;
  const mockOpen = open as Mock;
  const mockSave = save as Mock;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.resetAllMocks();
  });

  describe('verifyBundle', () => {
    it('should call invoke with correct parameters', async () => {
      const mockResponse = {
        status: 'ok',
        bundleId: 'test-uuid',
        manifestHash: '0x' + 'a'.repeat(64),
        proofHash: '0x' + 'b'.repeat(64),
        signatureValid: true,
        details: {
          manifest_hash: '0x' + 'a'.repeat(64),
          proof_hash: '0x' + 'b'.repeat(64),
          checks_passed: 5,
          checks_total: 5,
          statement_validation: [],
          signature_present: true,
        },
      };
      mockInvoke.mockResolvedValue(mockResponse);

      const request = { bundlePath: '/path/to/bundle.zip' };
      const result = await verifyBundle(request);

      expect(mockInvoke).toHaveBeenCalledWith('verify_bundle', { request });
      expect(result).toEqual(mockResponse);
    });

    it('should throw error with context on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Bundle not found'));

      await expect(verifyBundle({ bundlePath: '/invalid' })).rejects.toThrow(
        'Bundle verification failed: Bundle not found'
      );
    });

    it('should handle string errors', async () => {
      mockInvoke.mockRejectedValue('String error');

      await expect(verifyBundle({ bundlePath: '/invalid' })).rejects.toThrow(
        'Bundle verification failed: String error'
      );
    });
  });

  describe('getBundleInfo', () => {
    it('should call invoke with correct parameters', async () => {
      const mockInfo = {
        bundleId: 'test-uuid',
        schema: 'cap-bundle.v1',
        createdAt: '2025-12-14T10:00:00Z',
        proofUnits: [],
        fileCount: 5,
      };
      mockInvoke.mockResolvedValue(mockInfo);

      const result = await getBundleInfo('/path/to/bundle.zip');

      expect(mockInvoke).toHaveBeenCalledWith('get_bundle_info', { bundlePath: '/path/to/bundle.zip' });
      expect(result).toEqual(mockInfo);
    });

    it('should throw error with context on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Invalid bundle'));

      await expect(getBundleInfo('/invalid')).rejects.toThrow('Failed to load bundle info: Invalid bundle');
    });
  });

  describe('createProject', () => {
    it('should call invoke with workspace and name', async () => {
      const mockProject = {
        path: '/workspace/test-project',
        name: 'test-project',
        createdAt: '2025-12-14T10:00:00Z',
      };
      mockInvoke.mockResolvedValue(mockProject);

      const result = await createProject('/workspace', 'test-project');

      expect(mockInvoke).toHaveBeenCalledWith('create_project', { workspace: '/workspace', name: 'test-project' });
      expect(result).toEqual(mockProject);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Permission denied'));

      await expect(createProject('/workspace', 'test')).rejects.toThrow('Failed to create project: Permission denied');
    });
  });

  describe('listProjects', () => {
    it('should call invoke with workspace', async () => {
      const mockProjects = [
        { path: '/workspace/proj1', name: 'proj1', createdAt: '2025-12-14T10:00:00Z' },
        { path: '/workspace/proj2', name: 'proj2', createdAt: '2025-12-14T11:00:00Z' },
      ];
      mockInvoke.mockResolvedValue(mockProjects);

      const result = await listProjects('/workspace');

      expect(mockInvoke).toHaveBeenCalledWith('list_projects', { workspace: '/workspace' });
      expect(result).toEqual(mockProjects);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Directory not found'));

      await expect(listProjects('/invalid')).rejects.toThrow('Failed to list projects: Directory not found');
    });
  });

  describe('listAllProjects', () => {
    it('should call invoke without parameters', async () => {
      const mockProjects = [{ path: '/workspace/proj1', name: 'proj1', createdAt: '2025-12-14T10:00:00Z' }];
      mockInvoke.mockResolvedValue(mockProjects);

      const result = await listAllProjects();

      expect(mockInvoke).toHaveBeenCalledWith('list_all_projects');
      expect(result).toEqual(mockProjects);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Workspace not configured'));

      await expect(listAllProjects()).rejects.toThrow('Failed to list projects: Workspace not configured');
    });
  });

  describe('createNewProject', () => {
    it('should call invoke without parameters', async () => {
      const mockProject = { path: '/workspace/new-project', name: 'new-project', createdAt: '2025-12-14T10:00:00Z' };
      mockInvoke.mockResolvedValue(mockProject);

      const result = await createNewProject();

      expect(mockInvoke).toHaveBeenCalledWith('create_new_project');
      expect(result).toEqual(mockProject);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Cannot create'));

      await expect(createNewProject()).rejects.toThrow('Failed to create project: Cannot create');
    });
  });

  describe('createTempProject', () => {
    it('should call invoke without parameters', async () => {
      const mockProject = { path: '/tmp/cap-temp', name: 'cap-temp', createdAt: '2025-12-14T10:00:00Z' };
      mockInvoke.mockResolvedValue(mockProject);

      const result = await createTempProject();

      expect(mockInvoke).toHaveBeenCalledWith('create_temp_project');
      expect(result).toEqual(mockProject);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Temp creation failed'));

      await expect(createTempProject()).rejects.toThrow('Failed to create temp project: Temp creation failed');
    });
  });

  describe('createProjectInFolder', () => {
    it('should call invoke with folder parameter', async () => {
      const mockProject = { path: '/custom/folder', name: 'folder', createdAt: '2025-12-14T10:00:00Z' };
      mockInvoke.mockResolvedValue(mockProject);

      const result = await createProjectInFolder('/custom/folder');

      expect(mockInvoke).toHaveBeenCalledWith('create_project_in_folder', { folder: '/custom/folder' });
      expect(result).toEqual(mockProject);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Folder not writable'));

      await expect(createProjectInFolder('/readonly')).rejects.toThrow('Failed to create project: Folder not writable');
    });
  });

  describe('getProjectStatus', () => {
    it('should call invoke with project path', async () => {
      const mockStatus = {
        info: { path: '/project', name: 'project', createdAt: '2025-12-14T10:00:00Z' },
        hasSuppliersCSv: true,
        hasUbosCsv: false,
        hasPolicy: true,
        hasCommitments: true,
        hasManifest: false,
        hasProof: false,
        currentStep: 'manifest',
      };
      mockInvoke.mockResolvedValue(mockStatus);

      const result = await getProjectStatus('/project');

      expect(mockInvoke).toHaveBeenCalledWith('get_project_status', { project: '/project' });
      expect(result).toEqual(mockStatus);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Project not found'));

      await expect(getProjectStatus('/invalid')).rejects.toThrow('Failed to get project status: Project not found');
    });
  });

  describe('importCsv', () => {
    it('should call invoke with correct parameters', async () => {
      const mockResult = {
        csv_type: 'suppliers',
        record_count: 100,
        hash: '0x' + 'a'.repeat(64),
        destination: '/project/input/suppliers.csv',
      };
      mockInvoke.mockResolvedValue(mockResult);

      const result = await importCsv('/project', 'suppliers', '/path/to/file.csv');

      expect(mockInvoke).toHaveBeenCalledWith('import_csv', {
        project: '/project',
        csvType: 'suppliers',
        filePath: '/path/to/file.csv',
      });
      expect(result).toEqual(mockResult);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Invalid CSV format'));

      await expect(importCsv('/project', 'suppliers', '/bad.csv')).rejects.toThrow('Failed to import CSV: Invalid CSV format');
    });
  });

  describe('createCommitments', () => {
    it('should call invoke with project path', async () => {
      const mockResult = {
        supplier_root: '0x' + 'a'.repeat(64),
        ubo_root: '0x' + 'b'.repeat(64),
        company_root: '0x' + 'c'.repeat(64),
        path: '/project/build/commitments.json',
      };
      mockInvoke.mockResolvedValue(mockResult);

      const result = await createCommitments('/project');

      expect(mockInvoke).toHaveBeenCalledWith('create_commitments', { project: '/project' });
      expect(result).toEqual(mockResult);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Missing CSV files'));

      await expect(createCommitments('/project')).rejects.toThrow('Failed to create commitments: Missing CSV files');
    });
  });

  describe('loadPolicy', () => {
    it('should call invoke with correct parameters', async () => {
      const mockPolicy = {
        name: 'LkSG Policy',
        version: 'lksg.v1',
        hash: '0x' + 'a'.repeat(64),
        rules_count: 5,
        path: '/project/input/policy.yml',
      };
      mockInvoke.mockResolvedValue(mockPolicy);

      const result = await loadPolicy('/project', '/path/to/policy.yml');

      expect(mockInvoke).toHaveBeenCalledWith('load_policy', {
        project: '/project',
        policyPath: '/path/to/policy.yml',
      });
      expect(result).toEqual(mockPolicy);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Invalid policy format'));

      await expect(loadPolicy('/project', '/bad.yml')).rejects.toThrow('Failed to load policy: Invalid policy format');
    });
  });

  describe('buildManifest', () => {
    it('should call invoke with project path', async () => {
      const mockResult = {
        manifest_hash: '0x' + 'a'.repeat(64),
        path: '/project/build/manifest.json',
        supplier_root: '0x' + 'b'.repeat(64),
        ubo_root: '0x' + 'c'.repeat(64),
        policy_hash: '0x' + 'd'.repeat(64),
      };
      mockInvoke.mockResolvedValue(mockResult);

      const result = await buildManifest('/project');

      expect(mockInvoke).toHaveBeenCalledWith('build_manifest', { project: '/project' });
      expect(result).toEqual(mockResult);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Missing commitments'));

      await expect(buildManifest('/project')).rejects.toThrow('Failed to build manifest: Missing commitments');
    });
  });

  describe('buildProof', () => {
    it('should call invoke with project path', async () => {
      const mockResult = {
        proof_hash: '0x' + 'a'.repeat(64),
        path: '/project/build/proof.dat',
        backend: 'mock',
      };
      mockInvoke.mockResolvedValue(mockResult);

      const result = await buildProof('/project');

      expect(mockInvoke).toHaveBeenCalledWith('build_proof', { project: '/project' });
      expect(result).toEqual(mockResult);
    });

    it('should set up progress listener when callback provided', async () => {
      const mockResult = { proof_hash: '0x' + 'a'.repeat(64), path: '/project/build/proof.dat', backend: 'mock' };
      const unlisten = vi.fn();
      mockInvoke.mockResolvedValue(mockResult);
      mockListen.mockResolvedValue(unlisten);

      const onProgress = vi.fn();
      await buildProof('/project', onProgress);

      expect(mockListen).toHaveBeenCalledWith('proof:progress', expect.any(Function));
      expect(unlisten).toHaveBeenCalled();
    });

    it('should call progress callback with event payload', async () => {
      const mockResult = { proof_hash: '0x' + 'a'.repeat(64), path: '/project/build/proof.dat', backend: 'mock' };
      let capturedCallback: (event: { payload: { percent: number; message: string } }) => void = () => {};
      mockInvoke.mockResolvedValue(mockResult);
      mockListen.mockImplementation((_event, cb) => {
        capturedCallback = cb;
        return Promise.resolve(vi.fn());
      });

      const onProgress = vi.fn();
      const promise = buildProof('/project', onProgress);

      // Simulate progress event
      capturedCallback({ payload: { percent: 50, message: 'Building...' } });

      await promise;
      expect(onProgress).toHaveBeenCalledWith({ percent: 50, message: 'Building...' });
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Proof generation failed'));

      await expect(buildProof('/project')).rejects.toThrow('Failed to build proof: Proof generation failed');
    });

    it('should clean up listener on error', async () => {
      const unlisten = vi.fn();
      mockInvoke.mockRejectedValue(new Error('Failed'));
      mockListen.mockResolvedValue(unlisten);

      await expect(buildProof('/project', vi.fn())).rejects.toThrow();
      expect(unlisten).toHaveBeenCalled();
    });
  });

  describe('exportBundle', () => {
    it('should call invoke with correct parameters', async () => {
      const mockResult = {
        bundle_path: '/output/bundle.zip',
        size_bytes: 12345,
        hash: '0x' + 'a'.repeat(64),
        files: ['manifest.json', 'proof.dat'],
      };
      mockInvoke.mockResolvedValue(mockResult);

      const result = await exportBundle('/project', '/output/bundle.zip');

      expect(mockInvoke).toHaveBeenCalledWith('export_bundle', {
        project: '/project',
        output: '/output/bundle.zip',
      });
      expect(result).toEqual(mockResult);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Cannot write to output'));

      await expect(exportBundle('/project', '/readonly/bundle.zip')).rejects.toThrow('Failed to export bundle: Cannot write to output');
    });
  });

  describe('getAuditLog', () => {
    it('should call invoke with correct parameters', async () => {
      const mockLog = {
        events: [],
        totalCount: 0,
        chainValid: true,
        offset: 0,
        limit: 100,
      };
      mockInvoke.mockResolvedValue(mockLog);

      const result = await getAuditLog('/project', 50, 10);

      expect(mockInvoke).toHaveBeenCalledWith('get_audit_log', {
        project: '/project',
        limit: 50,
        offset: 10,
      });
      expect(result).toEqual(mockLog);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Audit log not found'));

      await expect(getAuditLog('/project')).rejects.toThrow('Failed to get audit log: Audit log not found');
    });
  });

  describe('verifyAuditChain', () => {
    it('should call invoke with project path', async () => {
      const mockResult = {
        valid: true,
        verifiedCount: 10,
        errors: [],
        tailHash: '0x' + 'a'.repeat(64),
      };
      mockInvoke.mockResolvedValue(mockResult);

      const result = await verifyAuditChain('/project');

      expect(mockInvoke).toHaveBeenCalledWith('verify_audit_chain', { project: '/project' });
      expect(result).toEqual(mockResult);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Chain verification failed'));

      await expect(verifyAuditChain('/project')).rejects.toThrow('Failed to verify audit chain: Chain verification failed');
    });
  });

  describe('generateKeys', () => {
    it('should call invoke with correct parameters', async () => {
      const mockKey = {
        kid: '0123456789abcdef',
        signerName: 'Test Company',
        publicKeyPath: '/project/keys/pubkey.pem',
        fingerprint: 'abcd1234',
        createdAt: '2025-12-14T10:00:00Z',
      };
      mockInvoke.mockResolvedValue(mockKey);

      const result = await generateKeys('/project', 'Test Company');

      expect(mockInvoke).toHaveBeenCalledWith('generate_keys', {
        project: '/project',
        signerName: 'Test Company',
      });
      expect(result).toEqual(mockKey);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Key generation failed'));

      await expect(generateKeys('/project', 'Test')).rejects.toThrow('Failed to generate keys: Key generation failed');
    });
  });

  describe('listKeys', () => {
    it('should call invoke with project path', async () => {
      const mockKeys = [
        {
          kid: '0123456789abcdef',
          signerName: 'Test Company',
          publicKeyPath: '/project/keys/pubkey.pem',
          fingerprint: 'abcd1234',
          createdAt: '2025-12-14T10:00:00Z',
        },
      ];
      mockInvoke.mockResolvedValue(mockKeys);

      const result = await listKeys('/project');

      expect(mockInvoke).toHaveBeenCalledWith('list_keys', { project: '/project' });
      expect(result).toEqual(mockKeys);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Cannot read keys'));

      await expect(listKeys('/project')).rejects.toThrow('Failed to list keys: Cannot read keys');
    });
  });

  describe('signProjectManifest', () => {
    it('should call invoke with correct parameters', async () => {
      const mockResult = {
        success: true,
        signer: 'Test Company',
        signatureHash: '0xabcd...',
        signatureHex: '0x' + 'a'.repeat(128),
        manifestPath: '/project/build/manifest.json',
      };
      mockInvoke.mockResolvedValue(mockResult);

      const result = await signProjectManifest('/project', 'Test Company');

      expect(mockInvoke).toHaveBeenCalledWith('sign_project_manifest', {
        project: '/project',
        signerName: 'Test Company',
      });
      expect(result).toEqual(mockResult);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Key not found'));

      await expect(signProjectManifest('/project', 'Unknown')).rejects.toThrow('Failed to sign manifest: Key not found');
    });
  });

  describe('verifyManifestSignature', () => {
    it('should call invoke with project path', async () => {
      const mockResult = {
        valid: true,
        signer: 'Test Company',
        algorithm: 'Ed25519',
      };
      mockInvoke.mockResolvedValue(mockResult);

      const result = await verifyManifestSignature('/project');

      expect(mockInvoke).toHaveBeenCalledWith('verify_manifest_signature', { project: '/project' });
      expect(result).toEqual(mockResult);
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Manifest not found'));

      await expect(verifyManifestSignature('/project')).rejects.toThrow('Failed to verify signature: Manifest not found');
    });
  });

  describe('getAppInfo', () => {
    it('should transform snake_case response to camelCase', async () => {
      mockInvoke.mockResolvedValue({
        workspace_path: '/home/user/CAP-Proofs',
        is_first_run: false,
        has_custom_path: true,
      });

      const result = await getAppInfo();

      expect(mockInvoke).toHaveBeenCalledWith('get_app_info');
      expect(result).toEqual({
        workspacePath: '/home/user/CAP-Proofs',
        isFirstRun: false,
        hasCustomPath: true,
      });
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Config not found'));

      await expect(getAppInfo()).rejects.toThrow('Failed to get app info: Config not found');
    });
  });

  describe('setWorkspacePath', () => {
    it('should call invoke with path', async () => {
      mockInvoke.mockResolvedValue('/custom/path');

      const result = await setWorkspacePath('/custom/path');

      expect(mockInvoke).toHaveBeenCalledWith('set_workspace_path', { path: '/custom/path' });
      expect(result).toBe('/custom/path');
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Invalid path'));

      await expect(setWorkspacePath('/invalid')).rejects.toThrow('Failed to set workspace path: Invalid path');
    });
  });

  describe('resetWorkspacePath', () => {
    it('should call invoke without parameters', async () => {
      mockInvoke.mockResolvedValue('/home/user/Documents/CAP-Proofs');

      const result = await resetWorkspacePath();

      expect(mockInvoke).toHaveBeenCalledWith('reset_workspace_path');
      expect(result).toBe('/home/user/Documents/CAP-Proofs');
    });

    it('should throw error on failure', async () => {
      mockInvoke.mockRejectedValue(new Error('Cannot reset'));

      await expect(resetWorkspacePath()).rejects.toThrow('Failed to reset workspace path: Cannot reset');
    });
  });

  describe('readFileContent', () => {
    it('should call invoke with correct parameters', async () => {
      mockInvoke.mockResolvedValue('file content here');

      const result = await readFileContent('/project', 'input/policy.yml');

      expect(mockInvoke).toHaveBeenCalledWith('read_file_content', {
        project: '/project',
        relativePath: 'input/policy.yml',
      });
      expect(result).toBe('file content here');
    });
  });

  describe('readPolicyContent', () => {
    it('should call readFileContent with policy path', async () => {
      mockInvoke.mockResolvedValue('policy: lksg.v1');

      const result = await readPolicyContent('/project');

      expect(mockInvoke).toHaveBeenCalledWith('read_file_content', {
        project: '/project',
        relativePath: 'input/policy.yml',
      });
      expect(result).toBe('policy: lksg.v1');
    });
  });

  describe('readManifestContent', () => {
    it('should call readFileContent with manifest path', async () => {
      mockInvoke.mockResolvedValue('{"version": "manifest.v0"}');

      const result = await readManifestContent('/project');

      expect(mockInvoke).toHaveBeenCalledWith('read_file_content', {
        project: '/project',
        relativePath: 'build/manifest.json',
      });
      expect(result).toBe('{"version": "manifest.v0"}');
    });
  });
});

describe('tauri dialog functions', () => {
  const mockOpen = open as Mock;
  const mockSave = save as Mock;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('selectBundleFile', () => {
    it('should call open with zip filter', async () => {
      mockOpen.mockResolvedValue('/path/to/bundle.zip');

      const result = await selectBundleFile();

      expect(mockOpen).toHaveBeenCalledWith({
        multiple: false,
        directory: false,
        filters: [{ name: 'Bundle', extensions: ['zip'] }],
        title: 'Select Bundle ZIP File',
      });
      expect(result).toBe('/path/to/bundle.zip');
    });

    it('should return null when cancelled', async () => {
      mockOpen.mockResolvedValue(null);

      const result = await selectBundleFile();

      expect(result).toBeNull();
    });
  });

  describe('selectBundleDirectory', () => {
    it('should call open with directory option', async () => {
      mockOpen.mockResolvedValue('/path/to/bundle');

      const result = await selectBundleDirectory();

      expect(mockOpen).toHaveBeenCalledWith({
        multiple: false,
        directory: true,
        title: 'Select Bundle Directory',
      });
      expect(result).toBe('/path/to/bundle');
    });

    it('should return null when cancelled', async () => {
      mockOpen.mockResolvedValue(null);

      const result = await selectBundleDirectory();

      expect(result).toBeNull();
    });
  });

  describe('selectCsvFile', () => {
    it('should call open with csv filter', async () => {
      mockOpen.mockResolvedValue('/path/to/data.csv');

      const result = await selectCsvFile();

      expect(mockOpen).toHaveBeenCalledWith({
        multiple: false,
        directory: false,
        filters: [{ name: 'CSV', extensions: ['csv'] }],
        title: 'CSV-Datei auswählen',
      });
      expect(result).toBe('/path/to/data.csv');
    });
  });

  describe('selectPolicyFile', () => {
    it('should call open with yaml filter', async () => {
      mockOpen.mockResolvedValue('/path/to/policy.yml');

      const result = await selectPolicyFile();

      expect(mockOpen).toHaveBeenCalledWith({
        multiple: false,
        directory: false,
        filters: [{ name: 'Policy', extensions: ['yml', 'yaml'] }],
        title: 'Policy-Datei auswählen',
      });
      expect(result).toBe('/path/to/policy.yml');
    });
  });

  describe('selectExportPath', () => {
    it('should call save with zip filter and default name', async () => {
      mockSave.mockResolvedValue('/path/to/export.zip');

      const result = await selectExportPath();

      expect(mockSave).toHaveBeenCalledWith({
        filters: [{ name: 'Bundle', extensions: ['zip'] }],
        title: 'Bundle speichern unter',
        defaultPath: expect.stringMatching(/cap-bundle-\d{4}-\d{2}-\d{2}\.zip/),
      });
      expect(result).toBe('/path/to/export.zip');
    });
  });

  describe('selectWorkspace', () => {
    it('should call open with directory option', async () => {
      mockOpen.mockResolvedValue('/custom/workspace');

      const result = await selectWorkspace();

      expect(mockOpen).toHaveBeenCalledWith({
        multiple: false,
        directory: true,
        title: 'Workspace-Ordner auswählen',
      });
      expect(result).toBe('/custom/workspace');
    });
  });

  describe('selectWorkingFolder', () => {
    it('should call open with directory option', async () => {
      mockOpen.mockResolvedValue('/working/folder');

      const result = await selectWorkingFolder();

      expect(mockOpen).toHaveBeenCalledWith({
        multiple: false,
        directory: true,
        title: 'Arbeitsordner für Proof auswählen',
      });
      expect(result).toBe('/working/folder');
    });
  });

  describe('selectWorkspaceFolder', () => {
    it('should call open with directory option', async () => {
      mockOpen.mockResolvedValue('/workspace/folder');

      const result = await selectWorkspaceFolder();

      expect(mockOpen).toHaveBeenCalledWith({
        multiple: false,
        directory: true,
        title: 'Speicherort für Proofs auswählen',
      });
      expect(result).toBe('/workspace/folder');
    });
  });
});

describe('tauri type exports', () => {
  it('should export all required types', async () => {
    const module = await import('../../lib/tauri');
    // Type checks via the module import
    expect(module).toBeDefined();

    // Verify functions are exported
    expect(typeof module.verifyBundle).toBe('function');
    expect(typeof module.getBundleInfo).toBe('function');
    expect(typeof module.createProject).toBe('function');
    expect(typeof module.listProjects).toBe('function');
    expect(typeof module.importCsv).toBe('function');
    expect(typeof module.createCommitments).toBe('function');
    expect(typeof module.loadPolicy).toBe('function');
    expect(typeof module.buildManifest).toBe('function');
    expect(typeof module.buildProof).toBe('function');
    expect(typeof module.exportBundle).toBe('function');
    expect(typeof module.getAuditLog).toBe('function');
    expect(typeof module.verifyAuditChain).toBe('function');
    expect(typeof module.generateKeys).toBe('function');
    expect(typeof module.listKeys).toBe('function');
    expect(typeof module.signProjectManifest).toBe('function');
    expect(typeof module.verifyManifestSignature).toBe('function');
  });
});
