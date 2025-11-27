/**
 * ProjectSidebar Component
 *
 * Displays workspace navigation with project list and creation dialog.
 */

import { useState, useEffect } from 'react';
import {
  listProjects,
  createProject,
  selectWorkspace,
  type ProjectInfo,
} from '../../lib/tauri';

interface ProjectSidebarProps {
  workspacePath: string | null;
  onWorkspaceChange: (path: string) => void;
  currentProject: ProjectInfo | null;
  onProjectSelect: (project: ProjectInfo) => void;
}

export function ProjectSidebar({
  workspacePath,
  onWorkspaceChange,
  currentProject,
  onProjectSelect,
}: ProjectSidebarProps) {
  const [projects, setProjects] = useState<ProjectInfo[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [newProjectName, setNewProjectName] = useState('');
  const [creating, setCreating] = useState(false);

  useEffect(() => {
    if (workspacePath) {
      loadProjects();
    }
  }, [workspacePath]);

  async function loadProjects() {
    if (!workspacePath) return;

    setLoading(true);
    setError(null);

    try {
      const projectList = await listProjects(workspacePath);
      setProjects(projectList);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }

  async function handleSelectWorkspace() {
    try {
      const selected = await selectWorkspace();
      if (selected) {
        onWorkspaceChange(selected);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleCreateProject() {
    if (!workspacePath || !newProjectName.trim()) return;

    setCreating(true);
    setError(null);

    try {
      const newProject = await createProject(workspacePath, newProjectName.trim());
      setProjects((prev) => [newProject, ...prev]);
      setShowCreateDialog(false);
      setNewProjectName('');
      onProjectSelect(newProject);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setCreating(false);
    }
  }

  return (
    <div className="w-64 bg-gray-50 border-r border-gray-200 flex flex-col h-full">
      {/* Workspace Header */}
      <div className="p-3 border-b border-gray-200">
        <div className="flex items-center justify-between mb-2">
          <span className="text-xs font-medium text-gray-500 uppercase tracking-wide">
            Workspace
          </span>
          <button
            onClick={handleSelectWorkspace}
            className="text-xs text-blue-600 hover:text-blue-800"
          >
            Wechseln
          </button>
        </div>
        {workspacePath ? (
          <div className="text-sm text-gray-700 truncate" title={workspacePath}>
            {workspacePath.split('/').pop() || workspacePath}
          </div>
        ) : (
          <button
            onClick={handleSelectWorkspace}
            className="w-full text-left text-sm text-gray-500 hover:text-gray-700 py-1"
          >
            Workspace auswählen...
          </button>
        )}
      </div>

      {/* Projects List */}
      <div className="flex-1 overflow-y-auto">
        <div className="p-3">
          <div className="flex items-center justify-between mb-2">
            <span className="text-xs font-medium text-gray-500 uppercase tracking-wide">
              Projekte
            </span>
            {workspacePath && (
              <button
                onClick={() => setShowCreateDialog(true)}
                className="text-blue-600 hover:text-blue-800"
                title="Neues Projekt"
              >
                <svg className="w-4 h-4" width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                </svg>
              </button>
            )}
          </div>

          {error && (
            <div className="text-xs text-red-600 mb-2 p-2 bg-red-50 rounded">
              {error}
            </div>
          )}

          {loading ? (
            <div className="flex items-center justify-center py-4">
              <div className="animate-spin rounded-full h-5 w-5 border-b-2 border-blue-600"></div>
            </div>
          ) : !workspacePath ? (
            <div className="text-sm text-gray-400 text-center py-4">
              Bitte wählen Sie einen Workspace
            </div>
          ) : projects.length === 0 ? (
            <div className="text-sm text-gray-400 text-center py-4">
              Keine Projekte vorhanden
            </div>
          ) : (
            <div className="space-y-1">
              {projects.map((project) => (
                <button
                  key={project.path}
                  onClick={() => onProjectSelect(project)}
                  className={`w-full text-left px-3 py-2 rounded-lg transition-colors ${
                    currentProject?.path === project.path
                      ? 'bg-blue-100 text-blue-800'
                      : 'hover:bg-gray-100 text-gray-700'
                  }`}
                >
                  <div className="flex items-center gap-2">
                    <svg className="w-4 h-4 flex-shrink-0" width="16" height="16" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    </svg>
                    <span className="truncate text-sm">{project.name}</span>
                  </div>
                  <div className="text-xs text-gray-400 mt-0.5 ml-6">
                    {formatDate(project.createdAt)}
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Create Project Dialog */}
      {showCreateDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl p-6 w-96">
            <h3 className="text-lg font-medium text-gray-900 mb-4">
              Neues Projekt erstellen
            </h3>
            <input
              type="text"
              value={newProjectName}
              onChange={(e) => setNewProjectName(e.target.value)}
              placeholder="Projektname"
              className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              autoFocus
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleCreateProject();
                if (e.key === 'Escape') setShowCreateDialog(false);
              }}
            />
            <div className="flex justify-end gap-2 mt-4">
              <button
                onClick={() => setShowCreateDialog(false)}
                className="px-4 py-2 text-gray-600 hover:text-gray-800"
                disabled={creating}
              >
                Abbrechen
              </button>
              <button
                onClick={handleCreateProject}
                disabled={creating || !newProjectName.trim()}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {creating ? 'Erstelle...' : 'Erstellen'}
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
    return date.toLocaleDateString('de-DE', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
    });
  } catch {
    return isoString;
  }
}

export default ProjectSidebar;
