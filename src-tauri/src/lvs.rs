use crate::executor;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct LvsOutput {
    report: Vec<LvsReport>,
    // LVM 경고/오류 메시지 객체 (log_type, log_message 등)
    #[serde(default)]
    log: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LvsReport {
    lv: Vec<LogicalVolume>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LogicalVolume {
    lv_name: String,
    vg_name: String,
    lv_attr: String,
    lv_size: String,
    pool_lv: String,
    origin: String,
    data_percent: Option<f64>,
    metadata_percent: Option<f64>,
    move_pv: String,
    mirror_log: String,
    copy_percent: Option<f64>,
    convert_lv: String,
    #[serde(default)]
    raid_sync_action: String,
    #[serde(default)]
    segtype: String,
    #[serde(default)]
    device_path: String,
    #[serde(default)]
    device_mapper: String,
    #[serde(default)]
    filesystems: Vec<Filesystem>,
}

pub fn parse_lvs_result(json_str: &str) -> Result<LvsOutput, String> {
    let mut output: LvsOutput = serde_json::from_str(json_str)
        .map_err(|err| format!("Failed to parse lvs output: {}\n{}", err, json_str))?;

    for report in &mut output.report {
        for lv in &mut report.lv {
            lv.device_path = format!("/dev/{}/{}", lv.vg_name, lv.lv_name);
            lv.device_mapper = format!("/dev/mapper/{}-{}", lv.vg_name, lv.lv_name);

            let mount_info = get_mount_info(&lv.device_mapper);
            lv.filesystems = mount_info.filesystems;
        }
    }
    Ok(output)
}

// findmnt --json -o  /dev/mapper/data_pool-mirror_logical_volume
#[derive(Deserialize)]
struct FindMntOutput {
    filesystems: Vec<Filesystem>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Filesystem {
    source: String,
    target: String,
    fstype: String,
    size: String,
    used: String,
    avail: String,
    #[serde(rename = "use%")]
    use_percent: String,
}

fn parse_get_mount_info_result(json_str: &str) -> FindMntOutput {
    // 마운트되지 않은 장치는 findmnt 가 아무것도 출력하지 않는다
    serde_json::from_str(json_str).unwrap_or(FindMntOutput {
        filesystems: Vec::new(),
    })
}

fn get_mount_info(device_mapper: &str) -> FindMntOutput {
    let json = executor::execute("findmnt", vec!["--json", "-o", "SOURCE,TARGET,FSTYPE,SIZE,USED,AVAIL,USE%", device_mapper]);
    parse_get_mount_info_result(&json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let json_str = r#"
 {
      "report": [
          {
              "lv": [
                  {"lv_name":"mirror_logical_volume", "vg_name":"data_pool", "lv_attr":"rwi-aor---", "lv_size":"1.70t", "pool_lv":"", "origin":"", "data_percent":null, "metadata_percent":null, "move_pv":"", "mirror_log":"", "copy_percent":23.49, "convert_lv":"", "raid_sync_action":"resync"}
              ]
          }
      ]
      ,
      "log": [
      ]
  }
       "#;
        let lvs_output = parse_lvs_result(json_str).unwrap();
        assert_eq!(lvs_output.report[0].lv.len(), 1);
    }

    #[test]
    fn test_parse_partial_lv_with_null_copy_percent() {
        let json_str = r#"
 {
      "report": [
          {
              "lv": [
                  {"lv_name":"mirror_logical_volume", "vg_name":"data_pool", "lv_attr":"rwi---r-p-", "lv_size":"1.70t", "pool_lv":"", "origin":"", "data_percent":null, "metadata_percent":null, "move_pv":"", "mirror_log":"", "copy_percent":null, "convert_lv":"", "raid_sync_action":""}
              ]
          }
      ]
      ,
      "log": [
          {"log_seq_num":1, "log_type":"warn", "log_context":"processing", "log_object_type":"vg", "log_object_name":"data_pool", "log_object_id":"", "log_object_group":"", "log_object_group_id":"", "log_message":"WARNING: VG data_pool is missing PV.", "log_errno":0, "log_ret_code":0}
      ]
  }
       "#;
        let lvs_output = parse_lvs_result(json_str).unwrap();
        assert_eq!(lvs_output.report[0].lv[0].copy_percent, None);
    }
    
    #[test]
    fn test_parse_get_mount_info() {
        let json_str = r#"
 {
   "filesystems": [
      {
         "source": "/dev/mapper/data_pool-mirror_logical_volume",
         "target": "/zelon-data",
         "fstype": "ext4",
         "size": "1.7T",
         "used": "9.9M",
         "avail": "1.6T",
         "use%": "5%"
      }
   ]
}

       "#;
        let lvs_output = parse_get_mount_info_result(json_str);
        assert_eq!(lvs_output.filesystems.len(), 1);
        assert_eq!(lvs_output.filesystems[0].use_percent, "5%");
    }
}
