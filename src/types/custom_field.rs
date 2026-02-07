//! Custom field types for the Asana API.

use serde::{Deserialize, Serialize};

use super::common::{Gid, StatusColor};

/// A custom field definition on a project or portfolio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomFieldDefinition {
    /// The unique identifier for the custom field.
    pub gid: Gid,
    /// The name of the custom field.
    pub name: String,
    /// The type of the custom field.
    pub resource_subtype: Option<CustomFieldType>,
    /// Whether the field is required.
    #[serde(default)]
    pub is_required: bool,
    /// The description of the custom field.
    pub description: Option<String>,
    /// The precision for number fields.
    pub precision: Option<u32>,
    /// The format for number fields (currency, percentage, etc.).
    pub format: Option<String>,
    /// The currency code for currency fields.
    pub currency_code: Option<String>,
    /// Enum options for enum/multi_enum fields.
    #[serde(default)]
    pub enum_options: Vec<EnumOption>,
}

/// The type of a custom field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustomFieldType {
    /// A text field.
    Text,
    /// A number field.
    Number,
    /// A single-select enum field.
    Enum,
    /// A multi-select enum field.
    MultiEnum,
    /// A date field.
    Date,
    /// A people field (user references).
    People,
    /// Unknown field type.
    #[serde(other)]
    Unknown,
}

/// An option for an enum custom field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumOption {
    /// The unique identifier for the option.
    pub gid: Gid,
    /// The display name of the option.
    pub name: String,
    /// Whether this option is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// The color of the option.
    pub color: Option<String>,
}

fn default_true() -> bool {
    true
}

/// A custom field setting on a project (links definition to project).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomFieldSetting {
    /// The unique identifier for the setting.
    pub gid: Gid,
    /// The custom field definition.
    pub custom_field: CustomFieldDefinition,
    /// Whether this field is important (shown prominently).
    #[serde(default)]
    pub is_important: bool,
}

/// A custom field value on a task or other resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomFieldValue {
    /// The unique identifier for the custom field.
    pub gid: Gid,
    /// The name of the custom field.
    pub name: Option<String>,
    /// The type of the custom field.
    pub resource_subtype: Option<CustomFieldType>,
    /// The display value (human-readable).
    pub display_value: Option<String>,
    /// Text value (for text fields).
    pub text_value: Option<String>,
    /// Number value (for number fields).
    pub number_value: Option<f64>,
    /// Selected enum option (for enum fields).
    pub enum_value: Option<EnumOption>,
    /// Selected enum options (for multi_enum fields).
    #[serde(default)]
    pub multi_enum_values: Vec<EnumOption>,
    /// Date value (for date fields).
    pub date_value: Option<DateValue>,
}

/// A date value for date custom fields.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateValue {
    /// The date in YYYY-MM-DD format.
    pub date: Option<String>,
    /// The datetime in ISO 8601 format.
    pub date_time: Option<String>,
}

/// Extracted status from a custom field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtractedStatus {
    /// The name of the status field.
    pub field_name: String,
    /// The current status value.
    pub value: Option<String>,
    /// The color of the status (if available from enum option).
    pub color: Option<String>,
    /// Mapped to standard status color.
    pub status_color: StatusColor,
}

/// Options for extracting a status field from custom fields.
#[derive(Debug, Clone, Default)]
pub struct StatusExtractionOptions {
    /// Exact field name to match (case-sensitive).
    pub exact_name: Option<String>,
    /// Field names containing this string (case-insensitive).
    pub contains: Option<String>,
}

/// Extract a status field from a list of custom field values.
///
/// Priority:
/// 1. Exact match for "Status" (case-sensitive)
/// 2. First field containing "status" (case-insensitive)
/// 3. First enum field if no status field found
///
/// Returns `None` if no suitable field is found.
pub fn extract_status_field(
    custom_fields: &[CustomFieldValue],
    options: Option<StatusExtractionOptions>,
) -> Option<ExtractedStatus> {
    let options = options.unwrap_or_default();

    let field = if let Some(exact_name) = &options.exact_name {
        custom_fields
            .iter()
            .find(|f| f.name.as_ref() == Some(exact_name))
    } else if let Some(contains) = &options.contains {
        let contains_lower = contains.to_lowercase();
        custom_fields.iter().find(|f| {
            f.name
                .as_ref()
                .map(|n| n.to_lowercase().contains(&contains_lower))
                .unwrap_or(false)
        })
    } else {
        // Default priority: exact "Status" match
        custom_fields
            .iter()
            .find(|f| f.name.as_deref() == Some("Status"))
            // Then any field containing "status" (case-insensitive)
            .or_else(|| {
                custom_fields.iter().find(|f| {
                    f.name
                        .as_ref()
                        .map(|n| n.to_lowercase().contains("status"))
                        .unwrap_or(false)
                })
            })
            // Finally, first enum field
            .or_else(|| {
                custom_fields
                    .iter()
                    .find(|f| matches!(f.resource_subtype, Some(CustomFieldType::Enum)))
            })
    };

    field.and_then(extract_from_field)
}

/// Extract status information from a single custom field.
fn extract_from_field(field: &CustomFieldValue) -> Option<ExtractedStatus> {
    let field_name = field.name.clone().unwrap_or_default();

    let (value, color, status_color) = match field.resource_subtype {
        Some(CustomFieldType::Enum) => {
            let value = field
                .enum_value
                .as_ref()
                .map(|e| e.name.clone())
                .or_else(|| field.display_value.clone());
            let color = field.enum_value.as_ref().and_then(|e| e.color.clone());
            let status_color = map_color_to_status(&color, &value);
            (value, color, status_color)
        }
        Some(CustomFieldType::Text) => {
            let value = field
                .text_value
                .clone()
                .or_else(|| field.display_value.clone());
            (value, None, StatusColor::None)
        }
        _ => (field.display_value.clone(), None, StatusColor::None),
    };

    Some(ExtractedStatus {
        field_name,
        value,
        color,
        status_color,
    })
}

/// Map an Asana color string and value to a standard status color.
fn map_color_to_status(color: &Option<String>, value: &Option<String>) -> StatusColor {
    // Try to map from color first
    if let Some(c) = color {
        let c_lower = c.to_lowercase();
        if c_lower.contains("green") {
            return StatusColor::Green;
        }
        if c_lower.contains("yellow") || c_lower.contains("orange") {
            return StatusColor::Yellow;
        }
        if c_lower.contains("red") {
            return StatusColor::Red;
        }
        if c_lower.contains("blue") {
            return StatusColor::Blue;
        }
    }

    // Try to infer from value
    if let Some(v) = value {
        let v_lower = v.to_lowercase();
        if v_lower.contains("on track")
            || v_lower.contains("complete")
            || v_lower.contains("done")
            || v_lower.contains("good")
        {
            return StatusColor::Green;
        }
        if v_lower.contains("at risk") || v_lower.contains("warning") || v_lower.contains("behind")
        {
            return StatusColor::Yellow;
        }
        if v_lower.contains("off track")
            || v_lower.contains("blocked")
            || v_lower.contains("critical")
        {
            return StatusColor::Red;
        }
    }

    StatusColor::None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_custom_field_definition() {
        let json = r#"{
            "gid": "123",
            "name": "Priority",
            "resource_subtype": "enum",
            "is_required": true,
            "enum_options": [
                {"gid": "1", "name": "High", "enabled": true, "color": "red"},
                {"gid": "2", "name": "Medium", "enabled": true, "color": "yellow"},
                {"gid": "3", "name": "Low", "enabled": true, "color": "green"}
            ]
        }"#;
        let field: CustomFieldDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(field.gid, "123");
        assert_eq!(field.name, "Priority");
        assert_eq!(field.resource_subtype, Some(CustomFieldType::Enum));
        assert!(field.is_required);
        assert_eq!(field.enum_options.len(), 3);
    }

    #[test]
    fn test_deserialize_custom_field_value() {
        let json = r#"{
            "gid": "456",
            "name": "Status",
            "resource_subtype": "enum",
            "display_value": "On Track",
            "enum_value": {
                "gid": "789",
                "name": "On Track",
                "enabled": true,
                "color": "green"
            }
        }"#;
        let value: CustomFieldValue = serde_json::from_str(json).unwrap();
        assert_eq!(value.gid, "456");
        assert_eq!(value.name, Some("Status".to_string()));
        assert!(value.enum_value.is_some());
    }

    #[test]
    fn test_extract_status_field_exact_match() {
        let fields = vec![
            CustomFieldValue {
                gid: "1".to_string(),
                name: Some("Priority".to_string()),
                resource_subtype: Some(CustomFieldType::Enum),
                display_value: Some("High".to_string()),
                text_value: None,
                number_value: None,
                enum_value: Some(EnumOption {
                    gid: "e1".to_string(),
                    name: "High".to_string(),
                    enabled: true,
                    color: Some("red".to_string()),
                }),
                multi_enum_values: vec![],
                date_value: None,
            },
            CustomFieldValue {
                gid: "2".to_string(),
                name: Some("Status".to_string()),
                resource_subtype: Some(CustomFieldType::Enum),
                display_value: Some("On Track".to_string()),
                text_value: None,
                number_value: None,
                enum_value: Some(EnumOption {
                    gid: "e2".to_string(),
                    name: "On Track".to_string(),
                    enabled: true,
                    color: Some("green".to_string()),
                }),
                multi_enum_values: vec![],
                date_value: None,
            },
        ];

        let status = extract_status_field(&fields, None).unwrap();
        assert_eq!(status.field_name, "Status");
        assert_eq!(status.value, Some("On Track".to_string()));
        assert_eq!(status.status_color, StatusColor::Green);
    }

    #[test]
    fn test_extract_status_field_contains() {
        let fields = vec![CustomFieldValue {
            gid: "1".to_string(),
            name: Some("Project Status".to_string()),
            resource_subtype: Some(CustomFieldType::Enum),
            display_value: Some("At Risk".to_string()),
            text_value: None,
            number_value: None,
            enum_value: Some(EnumOption {
                gid: "e1".to_string(),
                name: "At Risk".to_string(),
                enabled: true,
                color: Some("yellow".to_string()),
            }),
            multi_enum_values: vec![],
            date_value: None,
        }];

        let status = extract_status_field(&fields, None).unwrap();
        assert_eq!(status.field_name, "Project Status");
        assert_eq!(status.status_color, StatusColor::Yellow);
    }

    #[test]
    fn test_map_color_to_status() {
        assert_eq!(
            map_color_to_status(&Some("green".to_string()), &None),
            StatusColor::Green
        );
        assert_eq!(
            map_color_to_status(&Some("yellow-orange".to_string()), &None),
            StatusColor::Yellow
        );
        assert_eq!(
            map_color_to_status(&None, &Some("On Track".to_string())),
            StatusColor::Green
        );
        assert_eq!(
            map_color_to_status(&None, &Some("At Risk".to_string())),
            StatusColor::Yellow
        );
        assert_eq!(
            map_color_to_status(&None, &Some("Off Track".to_string())),
            StatusColor::Red
        );
    }
}
