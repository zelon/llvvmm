use crate::executor;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum DeviceType {
    Hdd,
    Ssd,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PvsOutput {
    report: Vec<PvsReport>,
    // LVM 경고/오류 메시지 객체 (log_type, log_message 등)
    #[serde(default)]
    log: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PvsReport {
    pv: Vec<PhysicalVolume>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PhysicalVolume {
    pv_name: String,
    vg_name: String,
    pv_fmt: String,
    pv_attr: String,
    pv_size: String,
    pv_free: String,
    #[serde(default)]
    device_model: String,
    #[serde(default)]
    device_type: Option<DeviceType>,
}

pub fn parse_pvs_result(json_str: &str) -> Result<PvsOutput, String> {
    let mut output: PvsOutput = serde_json::from_str(json_str)
        .map_err(|err| format!("Failed to parse pvs output: {}\n{}", err, json_str))?;

    // Set device_model and device_type for each PhysicalVolume
    for report in &mut output.report {
        for pv in &mut report.pv {
            let device_info = get_device_info(&pv.pv_name);
            pv.device_model = device_info.model;
            pv.device_type = device_info.device_type;
        }
    }
    Ok(output)
}

pub struct DeviceInfo {
    model: String,
    device_type: Option<DeviceType>,
}

pub fn parse_get_device_info_result(json_str: &str) -> DeviceInfo {
    #[derive(Deserialize)]
    struct LsblkOutput {
        blockdevices: Vec<BlockDevice>,
    }

    #[derive(Deserialize)]
    struct BlockDevice {
        model: Option<String>,
        #[serde(rename = "rota")]
        is_rotational: Option<bool>,
    }

    let blockdevices = serde_json::from_str::<LsblkOutput>(json_str)
        .map(|output| output.blockdevices)
        .unwrap_or_default();
    if let Some(device) = blockdevices.first() {
        let model = device
            .model
            .clone()
            .unwrap_or_else(|| "Unknown Model".to_string());
        let device_type = match device.is_rotational {
            Some(true) => Some(DeviceType::Hdd),
            Some(false) => Some(DeviceType::Ssd),
            None => None,
        };
        return DeviceInfo { model, device_type };
    }
    DeviceInfo {
        model: "Unknown Model".to_string(),
        device_type: None,
    }
}

fn get_device_info(device_name: &str) -> DeviceInfo {
    let json = executor::execute("lsblk", vec!["-J", "-o", "MODEL,ROTA", device_name]);
    parse_get_device_info_result(&json)
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
              "pv": [
                  {"pv_name":"/dev/sdb", "vg_name":"data_pool", "pv_fmt":"lvm2", "pv_attr":"a--", "pv_size":"<1.82t", "pv_free":"<345.78g"},
                  {"pv_name":"/dev/sdc", "vg_name":"data_pool", "pv_fmt":"lvm2", "pv_attr":"a--", "pv_size":"<1.82t", "pv_free":"122.21g"},
                  {"pv_name":"/dev/sdd", "vg_name":"data_pool", "pv_fmt":"lvm2", "pv_attr":"a--", "pv_size":"<223.57g", "pv_free":"0 "}
              ]
          }
      ]
      ,
      "log": [
      ]
  }
        "#;
        let pvs_output = parse_pvs_result(json_str).unwrap();
        assert_eq!(pvs_output.report[0].pv.len(), 3);
    }

    #[test]
    fn test_parse_with_log_messages() {
        let json_str = r#"
      {
      "report": [
          {
              "pv": [
                  {"pv_name":"/dev/sdb", "vg_name":"data_pool", "pv_fmt":"lvm2", "pv_attr":"a--", "pv_size":"<1.82t", "pv_free":"<345.78g"}
              ]
          }
      ]
      ,
      "log": [
          {"log_seq_num":1, "log_type":"warn", "log_context":"processing", "log_object_type":"pv", "log_object_name":"", "log_object_id":"", "log_object_group":"", "log_object_group_id":"", "log_message":"WARNING: test", "log_errno":0, "log_ret_code":1}
      ]
  }
        "#;
        let pvs_output = parse_pvs_result(json_str).unwrap();
        assert_eq!(pvs_output.report[0].pv.len(), 1);
        assert_eq!(pvs_output.log.len(), 1);
    }

    #[test]
    fn test_parse_get_device_info_result() {
        let json_str = r#"
{
   "blockdevices": [
      {
         "model": "Zelon",
         "rota": false
      },{
         "model": null,
         "rota": true
      },{
         "model": null,
         "rota": null
      }
   ]
}

        "#;
        let device_info = parse_get_device_info_result(json_str);
        assert_eq!(device_info.model, "Zelon");
        assert_eq!(device_info.device_type, Some(DeviceType::Ssd));
    }

    #[test]
    fn test_parse_get_device_info_result_hdd() {
        let json_str = r#"
{
   "blockdevices": [
      {
         "model": "Seagate",
         "rota": true
      }
   ]
}

        "#;
        let device_info = parse_get_device_info_result(json_str);
        assert_eq!(device_info.model, "Seagate");
        assert_eq!(device_info.device_type, Some(DeviceType::Hdd));
    }

}
