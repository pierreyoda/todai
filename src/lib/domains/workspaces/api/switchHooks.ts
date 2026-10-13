/**
 * Asked before switching to another workspace, e.g. to save today's note being written while its workspace is still the
 * active one: resolves to `false` to stop the switch.
 */
export type BeforeWorkspaceSwitch = () => Promise<boolean>;

/**
 * Called once switched to another workspace, along with refreshing what came from the former one's database: e.g. to
 * drop some of it instead. Resolves once done.
 */
export type AfterWorkspaceSwitch = () => Promise<unknown>;

const beforeHooks = new Set<BeforeWorkspaceSwitch>();
const afterHooks = new Set<AfterWorkspaceSwitch>();

/** Asks `hook` before each switch, until the returned function is called. */
export const beforeWorkspaceSwitch = (hook: BeforeWorkspaceSwitch): (() => void) => {
  beforeHooks.add(hook);
  return () => {
    beforeHooks.delete(hook);
  };
};

/** Asks the hooks in turn, stopping at the first refusal: whether to switch. */
export const canSwitchWorkspace = async (): Promise<boolean> => {
  for (const hook of [...beforeHooks]) {
    if (!(await hook())) return false;
  }
  return true;
};

/**
 * Calls `hook` after each switch, until the returned function is called. Lets the data of the higher domains (e.g. the
 * notes) react, without the workspaces knowing about them.
 */
export const afterWorkspaceSwitch = (hook: AfterWorkspaceSwitch): (() => void) => {
  afterHooks.add(hook);
  return () => {
    afterHooks.delete(hook);
  };
};

/** Calls the hooks at once: resolves once they're all done. */
export const runAfterWorkspaceSwitch = (): Promise<unknown[]> => Promise.all([...afterHooks].map((hook) => hook()));
