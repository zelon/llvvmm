use crate::executor;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PvsOutput {
    report: Vec<PvsReport>,
    log: Vec<String>,
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
}

pub fn parse_pvs_result(json_str: &str) -> PvsOutput {
    let mut output: PvsOutput = serde_json::from_str(&json_str).unwrap();
    
    // Set device_model for each PhysicalVolume
    for report in &mut output.report {
        for pv in &mut report.pv {
            pv.device_model = get_device_model(&pv.pv_name);
        }
    }
    output
}

pub fn parse_get_device_model_result(json_str: &str) -> String {
    #[derive(Deserialize)]
    struct LsblkOutput {
        blockdevices: Vec<BlockDevice>,
    }

    #[derive(Deserialize)]
    struct BlockDevice {
        model: Option<String>,
    }

    let output: LsblkOutput = serde_json::from_str(&json_str).unwrap();
    if let Some(device) = output.blockdevices.first() {
        if let Some(model) = &device.model {
            return model.clone();
        }
    }
    "Unknown Model".to_string()
}

fn get_device_model(device_name: &str) -> String {
    let json = executor::execute("lsblk", vec!["-J", "-o", "MODEL", device_name]);
    parse_get_device_model_result(&json)
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
        let pvs_output = parse_pvs_result(json_str);
        assert_eq!(pvs_output.report[0].pv.len(), 3);
    }

    #[test]
    fn test_parse_get_device_model_result() {
        let json_str = r#"
{
   "blockdevices": [
      {
         "model": "Zelon"
      },{
         "model": null
      },{
         "model": null
      }
   ]
}

        "#;
        let device_model = parse_get_device_model_result(json_str);
        assert_eq!(device_model, "Zelon");
    }

}
