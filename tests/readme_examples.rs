use cjson::{parse, to_string};

// Test ported from readme_examples.c — the monitor creation example

#[test]
fn create_and_print_monitor() {
    let expected = r#"{
  "name": "Awesome 4K",
  "resolutions": [
    {
      "height": 720,
      "width": 1280
    },
    {
      "height": 1080,
      "width": 1920
    },
    {
      "height": 2160,
      "width": 3840
    }
  ]
}"#;

    // Build the same JSON structure using our API
    let mut monitor = cjson::Value::object();
    monitor.insert("name", cjson::Value::string("Awesome 4K"));

    let mut resolutions = cjson::Value::array();

    let resolutions_data: [(u32, u32); 3] = [(1280, 720), (1920, 1080), (3840, 2160)];

    for (w, h) in resolutions_data {
        let mut resolution = cjson::Value::object();
        resolution.insert("width", cjson::Value::number(w as i64));
        resolution.insert("height", cjson::Value::number(h as i64));
        resolutions.push(resolution);
    }

    monitor.insert("resolutions", resolutions);

    let output = to_string(&monitor);
    assert_eq!(output, expected);
}

#[test]
fn parse_and_access_monitor() {
    let input = r#"{
  "name": "Awesome 4K",
  "resolutions": [
    { "width": 1280, "height": 720 },
    { "width": 1920, "height": 1080 },
    { "width": 3840, "height": 2160 }
  ]
}"#;

    let monitor = parse(input).expect("Failed to parse monitor JSON");

    // Check name
    let name = monitor.get("name").and_then(|v| v.as_str());
    assert_eq!(name, Some("Awesome 4K"));

    // Check resolutions
    let resolutions = monitor.get("resolutions").and_then(|v| v.as_array());
    assert!(resolutions.is_some());
    let resolutions = resolutions.unwrap();
    assert_eq!(resolutions.len(), 3);

    // Check full HD support
    let mut supports_full_hd = false;
    for resolution in resolutions {
        let width = resolution.get("width").and_then(|v| v.as_f64());
        let height = resolution.get("height").and_then(|v| v.as_f64());
        if width == Some(1920.0) && height == Some(1080.0) {
            supports_full_hd = true;
        }
    }
    assert!(supports_full_hd);
}

#[test]
fn supports_full_hd_should_check() {
    let json_with_hd = r#"{
  "name": "Awesome 4K",
  "resolutions": [
    { "width": 1280, "height": 720 },
    { "width": 1920, "height": 1080 }
  ]
}"#;

    let json_without_hd = r#"{
  "name": "lame monitor",
  "resolutions": [
    { "width": 640, "height": 480 }
  ]
}"#;

    let check = |json: &str| -> bool {
        let monitor = parse(json).unwrap();
        let resolutions = monitor.get("resolutions").unwrap().as_array().unwrap();
        for resolution in resolutions {
            let w = resolution.get("width").and_then(|v| v.as_f64());
            let h = resolution.get("height").and_then(|v| v.as_f64());
            if w == Some(1920.0) && h == Some(1080.0) {
                return true;
            }
        }
        false
    };

    assert!(check(json_with_hd));
    assert!(!check(json_without_hd));
}
