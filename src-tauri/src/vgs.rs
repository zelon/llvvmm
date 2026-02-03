
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VgsOutput {
    report: Vec<VgsReport>,
    log: Vec<String>,
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
}

pub fn parse_vgs_result(json_str: &str) -> VgsOutput {
    let output: VgsOutput = serde_json::from_str(&json_str).unwrap();
    output
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
        let vgs_output = parse_vgs_result(json_str);
        assert_eq!(vgs_output.report[0].vg.len(), 1);
    }
}
