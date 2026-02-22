import { invoke } from '@tauri-apps/api/core';
import type {
  SyncConnectionStatus,
  AccountInfo,
  DeviceInfo,
  SyncProjectState,
  PairingCode,
  SyncStatus,
} from '$lib/types';
import { StaleGuard } from './stale-guard';

/** Map a Rust SyncStatus enum to the frontend connection status string. */
function mapSyncStatus(status: SyncStatus): SyncConnectionStatus {
  if (status === 'Connected') return 'connected';
  if (status === 'Connecting') return 'connecting';
  if (status === 'Disconnected') return 'disconnected';
  if (typeof status === 'object' && 'Reconnecting' in status) return 'reconnecting';
  if (typeof status === 'object' && 'Error' in status) return 'error';
  return 'disconnected';
}

/** Extract the error message from a SyncStatus, if any. */
function extractError(status: SyncStatus): string | null {
  if (typeof status === 'object' && 'Error' in status) return status.Error.message;
  return null;
}

class SyncStore {
  connectionStatus = $state<SyncConnectionStatus>('disconnected');
  lastError = $state<string | null>(null);
  account = $state<AccountInfo | null>(null);
  devices = $state<DeviceInfo[]>([]);
  syncedProjects = $state<Map<string, SyncProjectState>>(new Map());
  pendingUpdates = $state(0);
  serverUrl = $state('');

  isLoggedIn = $derived(this.account !== null);
  isConnected = $derived(this.connectionStatus === 'connected');
  isSyncing = $derived(this.connectionStatus === 'connecting' || this.connectionStatus === 'reconnecting');

  private guard = new StaleGuard();

  /** Connect to the sync server. */
  async connect(serverUrl: string, jwtToken: string, deviceId: string): Promise<void> {
    const token = this.guard.snapshot();
    this.connectionStatus = 'connecting';
    this.lastError = null;
    try {
      await invoke('sync_connect', {
        serverUrl,
        jwtToken,
        deviceId,
      });
      if (this.guard.isStale(token)) return;
      this.connectionStatus = 'connected';
    } catch (e) {
      if (this.guard.isStale(token)) return;
      this.connectionStatus = 'error';
      this.lastError = String(e);
      throw e;
    }
  }

  /** Disconnect from the sync server. */
  async disconnect(): Promise<void> {
    const token = this.guard.snapshot();
    try {
      await invoke('sync_disconnect');
    } finally {
      if (!this.guard.isStale(token)) {
        this.connectionStatus = 'disconnected';
        this.lastError = null;
      }
    }
  }

  /** Fetch current sync status from the backend. */
  async refreshStatus(): Promise<void> {
    const token = this.guard.snapshot();
    try {
      const status = await invoke<SyncStatus>('sync_status');
      if (this.guard.isStale(token)) return;
      this.connectionStatus = mapSyncStatus(status);
      this.lastError = extractError(status);
    } catch (e) {
      if (this.guard.isStale(token)) return;
      this.lastError = String(e);
    }
  }

  /** Enable sync for a project. */
  async enableProjectSync(projectId: string, docKeyBytes: number[]): Promise<void> {
    const token = this.guard.snapshot();
    await invoke('sync_enable_project', {
      projectId,
      docKeyBytes,
    });
    if (this.guard.isStale(token)) return;
    this.syncedProjects.set(projectId, {
      projectId,
      enabled: true,
      lastSyncTime: null,
      pendingUpdates: 0,
    });
    // Trigger reactivity by reassigning the map
    this.syncedProjects = new Map(this.syncedProjects);
  }

  /** Disable sync for a project. */
  async disableProjectSync(projectId: string): Promise<void> {
    const token = this.guard.snapshot();
    await invoke('sync_disable_project', { projectId });
    if (this.guard.isStale(token)) return;
    this.syncedProjects.delete(projectId);
    this.syncedProjects = new Map(this.syncedProjects);
  }

  /** Send a CRDT update for a project. */
  async sendUpdate(projectId: string, updateBytes: number[]): Promise<void> {
    await invoke('sync_send_update', { projectId, updateBytes });
  }

  /** Generate a pairing code for this device. */
  async generatePairingCode(serverUrl: string): Promise<PairingCode> {
    return invoke<PairingCode>('generate_pairing_code', { serverUrl });
  }

  /** Complete pairing with a remote device. */
  async completePairing(remotePairingCode: string): Promise<DeviceInfo> {
    const token = this.guard.snapshot();
    const device = await invoke<DeviceInfo>('complete_pairing', { remotePairingCode });
    if (!this.guard.isStale(token)) {
      this.devices = [...this.devices, device];
    }
    return device;
  }

  /** List all paired devices (including this one). */
  async listDevices(): Promise<DeviceInfo[]> {
    const token = this.guard.snapshot();
    const devices = await invoke<DeviceInfo[]>('list_paired_devices');
    if (!this.guard.isStale(token)) {
      this.devices = devices;
    }
    return devices;
  }

  /** Remove a paired device (triggers key rotation). */
  async removeDevice(deviceId: string): Promise<void> {
    const token = this.guard.snapshot();
    await invoke('remove_device', { deviceId });
    if (this.guard.isStale(token)) return;
    this.devices = this.devices.filter((d) => d.device_id !== deviceId);
  }

  /** Handle a sync status change event from the backend. */
  handleStatusChanged(status: SyncStatus): void {
    this.connectionStatus = mapSyncStatus(status);
    this.lastError = extractError(status);
  }

  /** Handle a sync update received event from the backend. */
  handleUpdateReceived(projectId: string, _updateBytes: number[]): void {
    const project = this.syncedProjects.get(projectId);
    if (project) {
      project.lastSyncTime = new Date().toISOString();
      this.syncedProjects = new Map(this.syncedProjects);
    }
  }

  /** Log in with account info (called after magic link verification). */
  login(account: AccountInfo): void {
    this.account = account;
  }

  /** Log out and clear account state. */
  logout(): void {
    this.account = null;
    this.devices = [];
    this.syncedProjects = new Map();
    this.pendingUpdates = 0;
  }

  /** Reset all state. */
  reset(): void {
    this.connectionStatus = 'disconnected';
    this.lastError = null;
    this.account = null;
    this.devices = [];
    this.syncedProjects = new Map();
    this.pendingUpdates = 0;
    this.serverUrl = '';
    this.guard.reset();
  }
}

export const syncStore = new SyncStore();
