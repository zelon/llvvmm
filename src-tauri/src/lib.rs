mod pvs;

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_volume_group_status_string() {
        let c = 1 + 2;
        assert_eq!(c, 3);
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct VgsOutput {
    report: Vec<Report>,
    log: Vec<String>,  // 또는 Vec<LogEntry>
}

#[derive(Debug, Deserialize, Serialize)]
struct Report {
    vg: Vec<VolumeGroup>,
}

#[derive(Debug, Deserialize, Serialize)]
struct VolumeGroup {
    vg_name: String,
    pv_count: u32,
    lv_count: u32,
    snap_count: u32,
    vg_attr: String,
    vg_size: String,
    vg_free: String,
}

fn get_console_result_to_html(cmd: &str) -> String {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.to_string()
        })
        .unwrap_or_else(|err| format!("Failed to execute {}: {}", cmd, err))
}

fn get_command_result_to_json(cmd: &str) -> String {
    std::process::Command::new(cmd)
        .arg("--reportformat")
        .arg("json_std")
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.to_string()
        })
        .unwrap_or_else(|err| format!("Failed to execute {}: {}", cmd, err))
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_physical_volume_status() -> String {
    get_command_result_to_json("pvs")
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
fn get_logical_volume_status() -> String {
    get_console_result_to_html("lvdisplay")
}

#[tauri::command]
fn get_volume_group_status_string() -> String {
    let json_data = std::process::Command::new("vgs")
        .arg("--reportformat")
        .arg("json_std")
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.to_string()
        })
        .unwrap();

    // JSON 파싱
    let output: VgsOutput = serde_json::from_str(&json_data).unwrap();

    // 데이터 접근
    println!("Reports: {}", output.report.len());
    
    let mut volume_group_name = "Cannot find VG name".to_string();
    for report in &output.report {
        println!("\nVolume Groups in this report: {}", report.vg.len());
        
        for vg in &report.vg {
            volume_group_name = vg.vg_name.clone();
            println!("  VG Name: {}", vg.vg_name);
            println!("  PV Count: {}", vg.pv_count);
            println!("  LV Count: {}", vg.lv_count);
            println!("  Size: {}", vg.vg_size);
            println!("  Free: {}", vg.vg_free);
            println!("  Attributes: {}", vg.vg_attr);
        }
    }
    volume_group_name
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
            get_physical_volume_status,
            get_logical_volume_status,
            get_volume_group_status_string,
            get_volume_group_status_json,
            get_detail_physical_volume_status,
            get_ls_result])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
