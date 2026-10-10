const { invoke } = window.__TAURI__.core;

let pvStatusEl;
let lvStatusEl;
let vgStatusEl;

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

function getSyncStatus(lv) {
  const isRaidOrMirror = lv.lv_attr.charAt(0) === 'r' || lv.lv_attr.charAt(0) === 'm';
  if (!isRaidOrMirror) {
    return { label: '-', className: '' };
  }
  // lv_attr[8]: volume health, lv_attr[4]: state ('a' = active)
  const health = lv.lv_attr.charAt(8);
  if (health === 'p') {
    return { label: 'Degraded (missing PV)', className: 'sync-status-degraded' };
  }
  if (health === 'r' || health === 'm') {
    return { label: 'Needs Repair', className: 'sync-status-degraded' };
  }
  if (lv.lv_attr.charAt(4) !== 'a') {
    return { label: 'Inactive', className: '' };
  }
  const isSyncing = lv.raid_sync_action && lv.raid_sync_action !== 'idle';
  if (isSyncing) {
    return {
      label: `Syncing ${lv.copy_percent ?? '?'}% (${lv.raid_sync_action})`,
      className: 'sync-status-syncing',
    };
  }
  return { label: 'Fully Synced (Healthy)', className: 'sync-status-synced' };
}

function displayLogicalVolumeView(data) {
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
      ${data.report[0].lv.map(lv => {
        const syncStatus = getSyncStatus(lv);
        return `
        <tr>
          <td>${lv.lv_name}</td>
          <td>${lv.vg_name}</td>
          <td>${lv.device_path}</td>
          <td>${lv.device_mapper}</td>
          <td>${lv.lv_size}</td>
          <td class="${syncStatus.className}">${syncStatus.label}</td>
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
      `;
      }).join('')}
    </tbody>
  `;
  return table;
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
});
