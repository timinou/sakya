/**
 * Re-exports from loro-crdt for use in the application.
 *
 * Uses the base64 build which embeds WASM as base64 and initializes
 * synchronously — no vite-plugin-wasm or top-level-await needed.
 */
export { LoroDoc, LoroText } from 'loro-crdt/base64';
export type { Subscription } from 'loro-crdt/base64';
