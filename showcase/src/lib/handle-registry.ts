/**
 * HandleRegistry — tracks WASM handles and releases them in bulk.
 *
 * Usage:
 *   const registry = new HandleRegistry();
 *   const curve = registry.track(session.create_line(...));
 *   // later:
 *   registry.release(); // frees all tracked handles
 */
export class HandleRegistry {
  private owned = new Set<{ free(): void }>();

  /** Track a handle and return it (for inline use). */
  track<T extends { free(): void }>(handle: T): T {
    this.owned.add(handle);
    return handle;
  }

  /** Release and free all tracked handles. */
  release(): void {
    for (const h of this.owned) {
      try {
        h.free();
      } catch {
        // Handle may have already been freed; ignore.
      }
    }
    this.owned.clear();
  }

  /** Number of currently tracked handles. */
  get size(): number {
    return this.owned.size;
  }
}
