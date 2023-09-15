/**
 * Chains multiple handlers for a given event, calling them all
 * in order when the event is received.
 */
export function chain<T>(actions: ((event: T) => void)[]) {
  return (e: T) => {
    for (const action of actions) {
      action(e);
    }
  }
}

/**
 * Returns an event handler that triggers when a keyboard event for
 * the given key is received.
 */
export function onKey(key: string, action: () => void) {
  return (e: React.KeyboardEvent) => {
    if (e.key === key) {
      action()
    }
  }
}
