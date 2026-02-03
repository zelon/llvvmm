const { invoke } = window.__TAURI__.core;

let pvStatusEl;
let lvStatusEl;
let vgStatusEl;

function displayPhysicalVolumeView(data) {
  const table = document.createElement('table');
  table.innerHTML = `
    <thead>
      <tr>
        <th>PV Name</th>
        <th>Device Model</th>
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
      </tr>
    </thead>
    <tbody>
      ${data.report[0].vg.map(vg => `
        <tr>
          <td>${vg.vg_name}</td>
          <td>${vg.pv_count}</td>
          <td>${vg.vg_size}</td>
          <td>${vg.vg_free}</td>
        </tr>
      `).join('')}
    </tbody>
  `;
  return table;
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
      </tr>
    </thead>
    <tbody>
      ${data.report[0].lv.map(lv => `
        <tr>
          <td>${lv.lv_name}</td>
          <td>${lv.vg_name}</td>
          <td>${lv.device_path}</td>
          <td>${lv.device_mapper}</td>
          <td>${lv.lv_size}</td>
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
  return table;
}

async function refreshContent() {
  console.log(await invoke("get_detail_physical_volume_status"));
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  pvStatusEl.replaceChildren(displayPhysicalVolumeView(JSON.parse(await invoke("get_detail_physical_volume_status"))));
  vgStatusEl.replaceChildren(displayVolumeGroupView(JSON.parse(await invoke("get_detail_volume_group_status"))));
  lvStatusEl.replaceChildren(displayLogicalVolumeView(JSON.parse(await invoke("get_logical_volume_status"))));
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
