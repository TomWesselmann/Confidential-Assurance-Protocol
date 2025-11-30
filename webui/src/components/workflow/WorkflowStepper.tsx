/**
 * Workflow Stepper Component
 *
 * @description Visual navigation for the 6-step Prover workflow
 */

import {
  useWorkflowStore,
  STEP_ORDER,
  selectStepLabel,
  type WorkflowStep,
} from '../../store/workflowStore';

const STEP_ICONS: Record<WorkflowStep, string> = {
  import: '1',
  commitments: '2',
  policy: '3',
  manifest: '4',
  proof: '5',
  export: '6',
};

export const WorkflowStepper: React.FC = () => {
  const { currentStep, steps, canGoToStep, setCurrentStep } = useWorkflowStore();

  return (
    <nav aria-label="Workflow steps" className="bg-white dark:bg-gray-800 rounded px-2 py-1.5 border border-gray-200 dark:border-gray-700">
      <ol className="flex items-center justify-between">
        {STEP_ORDER.map((step, index) => {
          const isActive = step === currentStep;
          const isCompleted = steps[step].status === 'completed';
          const isError = steps[step].status === 'error';
          const canNavigate = canGoToStep(step);

          return (
            <li key={step} className="flex items-center flex-1">
              <button
                onClick={() => canNavigate && setCurrentStep(step)}
                disabled={!canNavigate}
                className={`flex items-center space-x-1 px-1.5 py-0.5 rounded transition-all w-full
                  ${canNavigate ? 'cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700' : 'cursor-not-allowed opacity-50'}
                  ${isActive ? 'bg-blue-50 dark:bg-blue-900/30' : ''}`}
                aria-current={isActive ? 'step' : undefined}
              >
                <div className={`w-5 h-5 rounded-full flex items-center justify-center flex-shrink-0 text-[10px] font-bold
                  ${isCompleted ? 'bg-green-500 text-white' : isError ? 'bg-red-500 text-white' : isActive ? 'bg-blue-600 text-white' : 'bg-gray-200 text-gray-600'}`}>
                  {isCompleted ? (
                    <svg className="w-3 h-3" width="12" height="12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                    </svg>
                  ) : isError ? (
                    <svg className="w-3 h-3" width="12" height="12" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  ) : (
                    STEP_ICONS[step]
                  )}
                </div>
                <span className={`text-[10px] font-medium truncate
                  ${isActive ? 'text-blue-600' : isCompleted ? 'text-green-600' : 'text-gray-500'}`}>
                  {selectStepLabel(step)}
                </span>
              </button>
              {index < STEP_ORDER.length - 1 && (
                <div className={`w-3 h-px flex-shrink-0 ${isCompleted ? 'bg-green-500' : 'bg-gray-300'}`} />
              )}
            </li>
          );
        })}
      </ol>
    </nav>
  );
};
