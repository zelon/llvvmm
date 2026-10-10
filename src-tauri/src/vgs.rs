
use crate::executor;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct VgsOutput {
    report: Vec<VgsReport>,
    // LVM 경고/오류 메시지 객체 (log_type, log_message 등)
    #[serde(default)]
    log: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct VgsReport {
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
    #[serde(default)]
    pv_names: String,
}

pub fn parse_vgs_result(json_str: &str) -> Result<VgsOutput, String> {
    let mut output: VgsOutput = serde_json::from_str(json_str)
        .map_err(|err| format!("Failed to parse vgs output: {}\n{}", err, json_str))?;

    let pv_names_by_vg = get_pv_names_by_vg();
    for report in &mut output.report {
        for vg in &mut report.vg {
            vg.pv_names = pv_names_by_vg.get(&vg.vg_name).cloned().unwrap_or_default();
        }
    }
    Ok(output)
}

fn get_pv_names_by_vg() -> HashMap<String, String> {
    let json_str = executor::execute(
        "pvs",
        vec!["--reportformat", "json_std", "-o", "vg_name,pv_name"],
    );
    parse_pv_names_by_vg(&json_str)
}

fn parse_pv_names_by_vg(json_str: &str) -> HashMap<String, String> {
    #[derive(Deserialize)]
    struct PvsOutput {
        report: Vec<PvsReport>,
    }

    #[derive(Deserialize)]
    struct PvsReport {
        pv: Vec<PhysicalVolumeRef>,
    }

    #[derive(Deserialize)]
    struct PhysicalVolumeRef {
        pv_name: String,
        vg_name: String,
    }

    let mut pv_names_by_vg: HashMap<String, Vec<String>> = HashMap::new();
    if let Ok(output) = serde_json::from_str::<PvsOutput>(json_str) {
        for report in output.report {
            for pv in report.pv {
                pv_names_by_vg
                    .entry(pv.vg_name)
                    .or_default()
                    .push(pv.pv_name);
            }
        }
    }

    pv_names_by_vg
        .into_iter()
        .map(|(vg_name, pv_names)| (vg_name, pv_names.join(", ")))
        .collect()
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
              "vg": [
                  {"vg_name":"data_pool", "pv_count":3, "lv_count":1, "snap_count":0, "vg_attr":"wz--n-", "vg_size":"<3.86t", "vg_free":"<467.99g"}
              ]
          }
      ]
      ,
      "log": [
      ]
  }
        "#;
        let vgs_output = parse_vgs_result(json_str).unwrap();
        assert_eq!(vgs_output.report[0].vg.len(), 1);
    }

    #[test]
    fn test_parse_pv_names_by_vg() {
        let json_str = r#"
 {
      "report": [
          {
              "pv": [
                  {"vg_name":"data_pool", "pv_name":"/dev/sdb"},
                  {"vg_name":"data_pool", "pv_name":"/dev/sdc"},
                  {"vg_name":"other_pool", "pv_name":"/dev/sdd"}
              ]
          }
      ]
      ,
      "log": [
      ]
  }
        "#;
        let pv_names_by_vg = parse_pv_names_by_vg(json_str);
        assert_eq!(
            pv_names_by_vg.get("data_pool").unwrap(),
            "/dev/sdb, /dev/sdc"
        );
        assert_eq!(pv_names_by_vg.get("other_pool").unwrap(), "/dev/sdd");
    }
}
