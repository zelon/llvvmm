const { invoke } = window.__TAURI__.core;

let pvStatusEl;
let lvStatusEl;
let vgStatusEl;

const SYNC_POLL_INTERVAL_MS = 10000;
let syncPollTimer = null;
const syncBaseline = {};

function clearSyncBaseline(lvName) {
  delete syncBaseline[lvName];
}

function formatDuration(totalSeconds) {
  const seconds = Math.round(totalSeconds);
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  if (minutes > 0) return `${minutes}m`;
  return `${seconds}s`;
}

// Anchors the first observed sample for an LV and measures progress against
// it, rather than consecutive 10s samples, since copy_percent barely moves
// within a single poll interval.
function estimateRemainingTime(lvName, currentPercent) {
  const now = Date.now();
  const baseline = syncBaseline[lvName];
  if (!baseline) {
    syncBaseline[lvName] = { percent: currentPercent, time: now };
    return null;
  }
  const elapsedSeconds = (now - baseline.time) / 1000;
  const percentDelta = currentPercent - baseline.percent;
  if (elapsedSeconds < 30 || percentDelta <= 0) {
    return null;
  }
  const remainingPercent = 100 - currentPercent;
  return (remainingPercent * elapsedSeconds) / percentDelta;
}

function formatDeviceType(deviceType) {
  return deviceType ? deviceType.toUpperCase() : 'Unknown';
}

function displayPhysicalVolumeView(data) {
  const table = document.createElement('table');
  table.innerHTML = `
    <thead>
      <tr>
        <th>PV Name</th>
        <th>Device Model</th>
        <th>Type</th>
        <th>VolumeGroup Name</th>
        <th>Size</th>
        <th>Free</th>
      </tr>
    </thead>
    <tbody>
      ${data.report[0].pv.map(pv => `
        <tr>
          <td>${pv.pv_name}</td>
          <td>${pv.device_model}</td>
          <td>${formatDeviceType(pv.device_type)}</td>
          <td>${pv.vg_name}</td>
          <td>${pv.pv_size}</td>
          <td>${pv.pv_free}</td>
        </tr>
      `).join('')}
    </tbody>
  `;
  return table;
}

function displayVolumeGroupView(data) {
  const table = document.createElement('table');
  table.innerHTML = `
    <thead>
      <tr>
        <th>VG Name</th>
        <th>PV Count</th>
        <th>VG Size</th>
        <th>VG Free</th>
        <th>PV Names</th>
      </tr>
    </thead>
    <tbody>
      ${data.report[0].vg.map(vg => `
        <tr>
          <td>${vg.vg_name}</td>
          <td>${vg.pv_count}</td>
          <td>${vg.vg_size}</td>
          <td>${vg.vg_free}</td>
          <td>${vg.pv_names}</td>
        </tr>
      `).join('')}
    </tbody>
  `;
  return table;
}

const NON_REDUNDANT_SEGTYPES = ['striped', 'raid0', 'raid0_meta'];

function getSyncStatus(lv) {
  const isRaidOrMirror = lv.lv_attr.charAt(0) === 'r' || lv.lv_attr.charAt(0) === 'm';
  const isNonRedundant = NON_REDUNDANT_SEGTYPES.includes(lv.segtype);
  if (!isRaidOrMirror && !isNonRedundant) {
    clearSyncBaseline(lv.lv_name);
    return { label: '-', className: '', isSyncing: false };
  }
  // lv_attr[8]: volume health, lv_attr[4]: state ('a' = active)
  const health = lv.lv_attr.charAt(8);
  if (health === 'p') {
    clearSyncBaseline(lv.lv_name);
    return { label: 'Degraded (missing PV)', className: 'sync-status-degraded', isSyncing: false };
  }
  if (health === 'r' || health === 'm') {
    clearSyncBaseline(lv.lv_name);
    return { label: 'Needs Repair', className: 'sync-status-degraded', isSyncing: false };
  }
  // striped/raid0 have no copies to sync
  if (isNonRedundant) {
    clearSyncBaseline(lv.lv_name);
    return { label: 'N/A', className: '', isSyncing: false };
  }
  if (lv.lv_attr.charAt(4) !== 'a') {
    clearSyncBaseline(lv.lv_name);
    return { label: 'Inactive', className: '', isSyncing: false };
  }
  const isSyncing = lv.raid_sync_action && lv.raid_sync_action !== 'idle';
  if (isSyncing) {
    const remainingSeconds = estimateRemainingTime(lv.lv_name, lv.copy_percent);
    const eta = remainingSeconds === null ? 'ETA calculating...' : `ETA ~${formatDuration(remainingSeconds)}`;
    return {
      label: `Syncing ${lv.copy_percent ?? '?'}% (${lv.raid_sync_action}) — ${eta}`,
      className: 'sync-status-syncing',
      isSyncing: true,
    };
  }
  clearSyncBaseline(lv.lv_name);
  return { label: 'Fully Synced (Healthy)', className: 'sync-status-synced', isSyncing: false };
}

function displayLogicalVolumeView(data) {
  const rows = data.report[0].lv.map(lv => ({ lv, syncStatus: getSyncStatus(lv) }));
  const table = document.createElement('table');
  table.innerHTML = `
    <thead>
      <tr>
        <th>LogicalVolume Name</th>
        <th>VolumeGroup Name</th>
        <th>Device Path</th>
        <th>Device Mapper</th>
        <th>LogicalVolume Size</th>
        <th>Sync Status</th>
      </tr>
    </thead>
    <tbody>
      ${rows.map(({ lv, syncStatus }) => `
        <tr>
          <td>${lv.lv_name}</td>
          <td>${lv.vg_name}</td>
          <td>${lv.device_path}</td>
          <td>${lv.device_mapper}</td>
          <td>${lv.lv_size}</td>
          <td id="sync-status-${lv.lv_name}" class="${syncStatus.className}">${syncStatus.label}</td>
          <td>
            <table>
              <thead>
                <tr>
                  <th>Target</th>
                  <th>Filesystem Type</th>
                  <th>Size</th>
                  <th>Used</th>
                  <th>Avail</th>
                  <th>Use%</th>
                </tr>
              </thead>
              <tbody>
                ${lv.filesystems.map(fs => `
                  <tr>
                    <td>${fs.target}</td>
                    <td>${fs.fstype}</td>
                    <td>${fs.size}</td>
                    <td>${fs.used}</td>
                    <td>${fs.avail}</td>
                    <td>${fs["use%"]}</td>
                  </tr>
                `).join('')}
              </tbody>
            </table>
          </td>
        </tr>
      `).join('')}
    </tbody>
  `;
  updateSyncPolling(rows.some(({ syncStatus }) => syncStatus.isSyncing));
  return table;
}

async function refreshSyncStatuses() {
  try {
    const data = JSON.parse(await invoke("get_logical_volume_status"));
    const rows = data.report[0].lv.map(lv => ({ lv, syncStatus: getSyncStatus(lv) }));
    rows.forEach(({ lv, syncStatus }) => {
      const cell = document.getElementById(`sync-status-${lv.lv_name}`);
      if (cell) {
        cell.textContent = syncStatus.label;
        cell.className = syncStatus.className;
      }
    });
    updateSyncPolling(rows.some(({ syncStatus }) => syncStatus.isSyncing));
  } catch (err) {
    console.error("get_logical_volume_status failed:", err);
  }
}

function updateSyncPolling(shouldPoll) {
  if (shouldPoll && !syncPollTimer) {
    syncPollTimer = setInterval(refreshSyncStatuses, SYNC_POLL_INTERVAL_MS);
  } else if (!shouldPoll && syncPollTimer) {
    clearInterval(syncPollTimer);
    syncPollTimer = null;
  }
}

async function refreshSection(el, command, display) {
  try {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    el.replaceChildren(display(JSON.parse(await invoke(command))));
  } catch (err) {
    console.error(`${command} failed:`, err);
    const errorEl = document.createElement("pre");
    errorEl.textContent = `${command} failed: ${err}`;
    el.replaceChildren(errorEl);
  }
}

async function refreshContent() {
  await refreshSection(pvStatusEl, "get_detail_physical_volume_status", displayPhysicalVolumeView);
  await refreshSection(vgStatusEl, "get_detail_volume_group_status", displayVolumeGroupView);
  await refreshSection(lvStatusEl, "get_logical_volume_status", displayLogicalVolumeView);
}

window.addEventListener("DOMContentLoaded", () => {
  pvStatusEl = document.querySelector("#pv-status");
  lvStatusEl = document.querySelector("#lv-status");
  vgStatusEl = document.querySelector("#vg-status");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    refreshContent();
  });
  refreshContent();
});
