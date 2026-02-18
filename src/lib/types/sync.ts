/** Connection status for the sync engine. */
export type SyncConnectionStatus =
  | 'connected'
  | 'connecting'
  | 'disconnected'
  | 'reconnecting'
  | 'error';

/** Account information for a logged-in user. */
export interface AccountInfo {
  email: string;
  accountId: string;
}

/** Information about a paired device. */
export interface DeviceInfo {
  device_id: string;
  name: string;
  is_current: boolean;
}

/** Sync state for an individual project. */
export interface SyncProjectState {
  projectId: string;
  enabled: boolean;
  lastSyncTime: string | null;
  pendingUpdates: number;
}

/** Pairing code returned from generate_pairing_code. */
export interface PairingCode {
  qr_svg: string;
  pairing_string: string;
}

/** Sync status as returned by the Rust backend. */
export type SyncStatus =
  | 'Connected'
  | 'Connecting'
  | 'Disconnected'
  | { Reconnecting: { attempt: number } }
  | { Error: { message: string } };
