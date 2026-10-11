/**
 * Asked before switching to another workspace, e.g. to save today's note being written while its workspace is still the
 * active one: resolves to `false` to stop the switch.
 */
export type BeforeWorkspaceSwitch = () => Promise<boolean>;

const hooks = new Set<BeforeWorkspaceSwitch>();

/** Asks `hook` before each switch, until the returned function is called. */
export const beforeWorkspaceSwitch = (hook: BeforeWorkspaceSwitch): (() => void) => {
  hooks.add(hook);
  return () => {
    hooks.delete(hook);
  };
};

/** Asks the hooks in turn, stopping at the first refusal: whether to switch. */
export const canSwitchWorkspace = async (): Promise<boolean> => {
  for (const hook of [...hooks]) {
    if (!(await hook())) return false;
  }
  return true;
};
