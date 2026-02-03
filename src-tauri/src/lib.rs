mod executor;
mod lvs;
mod pvs;
mod vgs;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_detail_physical_volume_status() -> String {
    let json_data = std::process::Command::new("pvs")
    .arg("--reportformat")
    .arg("json_std")
    .output()
    .map(|output| {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.to_string()
    })
    .unwrap();

    // JSON 파싱
    let output = pvs::parse_pvs_result(&json_data);
    serde_json::to_string(&output).unwrap()
}

#[tauri::command]
fn get_detail_volume_group_status() -> String {
    let vgs_json_result = crate::executor::execute("vgs", vec!["--reportformat", "json_std"]);
    let vgs_output = vgs::parse_vgs_result(&vgs_json_result);
    serde_json::to_string(&vgs_output).unwrap()
}

#[tauri::command]
fn get_logical_volume_status() -> String {
    let lvs_json_result = crate::executor::execute("lvs", vec!["--reportformat", "json_std"]);
    let lvs_output = lvs::parse_lvs_result(&lvs_json_result);
    serde_json::to_string(&lvs_output).unwrap()
}

#[tauri::command]
fn get_volume_group_status_json() -> String {
    let json_data = std::process::Command::new("vgs")
        .arg("--reportformat")
        .arg("json_std")
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.to_string()
        })
        .unwrap();
    json_data
}

#[tauri::command]
fn get_ls_result() -> String {
    // ls 를 shell 에서 실행해서 실행 결과를 반환한다
    std::process::Command::new("ls")
        .arg("-l")
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            format!("ls result from rust for :\n{}", stdout)
        })
        .unwrap_or_else(|err| format!("Failed to execute ls: {}", err))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet,
            get_logical_volume_status,
            get_volume_group_status_json,
            get_detail_physical_volume_status,
            get_detail_volume_group_status,
            get_ls_result])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
