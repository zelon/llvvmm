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
        <th>Size</th>
        <th>Device Model</th>
      </tr>
    </thead>
    <tbody>
      ${data.report[0].pv.map(pv => `
        <tr>
          <td>${pv.pv_name}</td>
          <td>${pv.pv_size}</td>
          <td>${pv.device_model}</td>
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


async function greet() {
  console.log(await invoke("get_detail_physical_volume_status"));
  // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
  pvStatusEl.replaceChildren(displayPhysicalVolumeView(JSON.parse(await invoke("get_detail_physical_volume_status"))));
  vgStatusEl.textContent = await invoke("get_volume_group_status_json");
  lvStatusEl.textContent = await invoke("get_logical_volume_status");
}

window.addEventListener("DOMContentLoaded", () => {
  pvStatusEl = document.querySelector("#pv-status");
  lvStatusEl = document.querySelector("#lv-status");
  vgStatusEl = document.querySelector("#vg-status");
  document.querySelector("#greet-form").addEventListener("submit", (e) => {
    e.preventDefault();
    greet();
  });
});
