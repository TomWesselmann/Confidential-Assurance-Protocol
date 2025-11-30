/**
 * SimpleSidebar Component
 *
 * Sidebar with workspace path display and change option.
 * Supports first-run setup and path customization.
 */

import { useState, useEffect } from 'react';
import {
  listAllProjects,
  createNewProject,
  getProjectStatus,
  getAppInfo,
  setWorkspacePath,
  selectWorkspaceFolder,
  type ProjectInfo,
  type AppInfo,
} from '../../lib/tauri';

interface SimpleSidebarProps {
  currentProject: ProjectInfo | null;
  onProjectSelect: (project: ProjectInfo) => void;
}

export function SimpleSidebar({
  currentProject,
  onProjectSelect,
}: SimpleSidebarProps) {
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [showFirstRunDialog, setShowFirstRunDialog] = useState(false);

  useEffect(() => {
    initializeApp();
  }, []);

  async function initializeApp() {
    setLoading(true);
    setError(null);

    try {
      const info = await getAppInfo();
      setAppInfo(info);

      // Show first-run dialog if needed
      if (info.isFirstRun) {
        setShowFirstRunDialog(true);
      }

      const projectList = await listAllProjects();
      setProjects(projectList);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function loadProjects() {
    setLoading(true);
    setError(null);

    try {
      const projectList = await listAllProjects();
      setProjects(projectList);
      // Refresh app info too
      const info = await getAppInfo();
      setAppInfo(info);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleCreateProject() {
    setCreating(true);
    setError(null);

    try {
      const newProject = await createNewProject();
      setProjects((prev) => [newProject, ...prev]);

      // Get full status and select the new project
      const status = await getProjectStatus(newProject.path);
      onProjectSelect({
        path: newProject.path,
        name: status.info.name,
        createdAt: status.info.createdAt,
      });
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setCreating(false);
    }
  }

  async function handleChangeWorkspace() {
    try {
      const selected = await selectWorkspaceFolder();
      if (selected) {
        await setWorkspacePath(selected);
        setShowFirstRunDialog(false);
        await loadProjects();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleAcceptDefault() {
    setShowFirstRunDialog(false);
    // Settings will be saved on first project creation
  }

  // Get shortened path for display
  function getShortPath(path: string): string {
    const home = path.replace(/^\/Users\/[^/]+/, '~');
    if (home.length > 30) {
      const parts = home.split('/');
      if (parts.length > 3) {
        return `${parts[0]}/.../${parts[parts.length - 1]}`;
      }
    }
    return home;
  }

  return (
    <div className="w-56 bg-gray-50 border-r border-gray-200 flex flex-col h-full overflow-hidden">
      {/* Header with New Project Button */}
      <div className="p-3 border-b border-gray-200">
        <button
          onClick={handleCreateProject}
          disabled={creating}
          className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors"
        >
          {creating ? (
            <>
              <div className="animate-spin rounded-full h-4 w-4 border-2 border-white border-t-transparent"></div>
              <span>Creating...</span>
            </>
          ) : (
            <>
              <svg className="w-4 h-4" width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
              </svg>
              <span>New Proof</span>
            </>
          )}
        </button>
      </div>

      {/* Projects List */}
      <div className="flex-1 overflow-y-auto min-h-0">
        <div className="p-2">
          <div className="text-xs font-medium text-gray-400 uppercase tracking-wide px-2 py-1">
            Projects
          </div>

          {error && (
            <div className="text-xs text-red-600 p-2 bg-red-50 rounded m-1">
              {error}
            </div>
          )}

          {loading ? (
            <div className="flex items-center justify-center py-8">
              <div className="animate-spin rounded-full h-5 w-5 border-2 border-blue-600 border-t-transparent"></div>
            </div>
          ) : projects.length === 0 ? (
            <div className="text-sm text-gray-400 text-center py-8 px-2">
              No projects yet.
              <br />
              <span className="text-xs">Click "New Proof" to start.</span>
            </div>
          ) : (
            <div className="space-y-0.5 mt-1">
              {projects.map((project) => (
                <button
                  key={project.path}
                  onClick={() => onProjectSelect(project)}
                  className={`w-full text-left px-2 py-1.5 rounded transition-colors ${
                    currentProject?.path === project.path
                      ? 'bg-blue-100 text-blue-800'
                      : 'hover:bg-gray-100 text-gray-700'
                  }`}
                >
                  <div className="flex items-center gap-2">
                    <svg className="w-3.5 h-3.5 flex-shrink-0 text-gray-400" width="14" height="14" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    </svg>
                    <span className="truncate text-sm">{project.name}</span>
                  </div>
                  <div className="text-xs text-gray-400 ml-5.5 pl-0.5">
                    {formatDate(project.createdAt)}
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Footer - Storage Location with Change Link */}
      <div className="flex-shrink-0 p-2 border-t border-gray-200 bg-gray-50">
        <div className="flex items-center justify-between text-xs">
          <span
            className="text-gray-400 truncate flex-1"
            title={appInfo?.workspacePath || ''}
          >
            {appInfo ? getShortPath(appInfo.workspacePath) : '...'}
          </span>
          <button
            onClick={handleChangeWorkspace}
            className="text-blue-500 hover:text-blue-700 ml-1 flex-shrink-0"
            title="Change storage location"
          >
            Change
          </button>
        </div>
      </div>

      {/* First Run Dialog */}
      {showFirstRunDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-md mx-4 p-6">
            <h2 className="text-lg font-semibold text-gray-900 mb-2">
              Welcome to CAP Desktop Prover
            </h2>
            <p className="text-sm text-gray-600 mb-4">
              Where should your proofs be stored?
            </p>

            <div className="bg-gray-50 rounded-lg p-3 mb-4">
              <div className="text-xs text-gray-500 mb-1">Current storage location:</div>
              <div className="text-sm font-mono text-gray-800 break-all">
                {appInfo?.workspacePath || '~/Documents/CAP-Proofs'}
              </div>
            </div>

            <div className="flex gap-3">
              <button
                onClick={handleChangeWorkspace}
                className="flex-1 px-4 py-2 text-sm border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
              >
                Choose different folder
              </button>
              <button
                onClick={handleAcceptDefault}
                className="flex-1 px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                OK, looks good
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function formatDate(isoString: string | undefined): string {
  if (!isoString) return '';
  try {
    const date = new Date(isoString);
    return date.toLocaleDateString('en-US', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
    });
  } catch {
    return isoString;
  }
}

export default SimpleSidebar;
