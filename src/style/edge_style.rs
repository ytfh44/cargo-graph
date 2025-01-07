pub struct EdgeStyle;

impl EdgeStyle {
    pub fn get_color_and_style(label: &str) -> (String, String) {
        match label {
            "是" => ("green".to_string(), "solid".to_string()),
            "否" => ("red".to_string(), "solid".to_string()),
            "继续循环" => ("blue".to_string(), "dashed".to_string()),
            "跳出循环" => ("red".to_string(), "dashed".to_string()),
            _ => ("black".to_string(), "solid".to_string()),
        }
    }
} 