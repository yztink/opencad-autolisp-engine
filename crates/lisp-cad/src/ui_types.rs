use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

/// A single interactive control inside a dialog row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DialogControl {
    InfoText {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align_h: Option<HorizontalAlign>,
    },
    Text {
        id: String,
        label: String,
        default: String,
    },
    Number {
        id: String,
        label: String,
        default: f64,
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
    },
    Checkbox {
        id: String,
        label: String,
        default: bool,
    },
    Choice {
        id: String,
        label: String,
        options: Vec<String>,
        default_index: usize,
    },
    Color {
        id: String,
        label: String,
        default: [u8; 3],
    },
    FilePicker {
        id: String,
        label: String,
        default_path: String,
        filter_name: String,
        extensions: Vec<String>,
        save: bool,
    },
    Fields {
        id: String,
        title: Option<String>,
        controls: Vec<DialogControl>,
        #[serde(default)]
        collapsed: bool,
    },
    Table {
        id: String,
        label: Option<String>,
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        #[serde(default = "default_true")]
        allow_add: bool,
        #[serde(default = "default_true")]
        allow_delete: bool,
    },
    Button {
        id: String,
        label: String,
        #[serde(default)]
        action: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align_h: Option<HorizontalAlign>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<f32>,
    },
    Row {
        controls: Vec<DialogControl>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align_h: Option<HorizontalAlign>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align_v: Option<VerticalAlign>,
    },
    Column {
        controls: Vec<DialogControl>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align_h: Option<HorizontalAlign>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        align_v: Option<VerticalAlign>,
    },
}

fn default_true() -> bool {
    true
}

/// A horizontal row of controls inside a dialog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogRow {
    pub controls: Vec<DialogControl>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align_h: Option<HorizontalAlign>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align_v: Option<VerticalAlign>,
}

impl DialogRow {
    pub fn new(controls: Vec<DialogControl>) -> Self {
        Self {
            controls,
            align_h: None,
            align_v: None,
        }
    }

    pub fn single(control: DialogControl) -> Self {
        Self {
            controls: vec![control],
            align_h: None,
            align_v: None,
        }
    }

    pub fn with_align_h(mut self, align: HorizontalAlign) -> Self {
        self.align_h = Some(align);
        self
    }

    pub fn with_align_v(mut self, align: VerticalAlign) -> Self {
        self.align_v = Some(align);
        self
    }
}

/// Definition of a modal dialog requested by a plugin or Lisp script.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DialogDef {
    pub title: String,
    pub rows: Vec<DialogRow>,
    pub submit_label: String,
    pub cancel_label: String,
    #[serde(default)]
    pub width: Option<f32>,
}

impl DialogDef {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            rows: Vec::new(),
            submit_label: "OK".to_string(),
            cancel_label: "Cancel".to_string(),
            width: None,
        }
    }
}

/// Definition of a persistent dockable side panel / palette.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelDef {
    pub id: String,
    pub title: String,
    pub controls: Vec<DialogControl>,
    #[serde(default)]
    pub width: Option<f32>,
    #[serde(default)]
    pub command: Option<String>,
}

impl PanelDef {
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            controls: Vec::new(),
            width: None,
            command: None,
        }
    }
}

/// Value returned by or updated on an interactive control.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DialogValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Choice { index: usize, text: String },
    Color([u8; 3]),
    FilePath(String),
    Fields(HashMap<String, DialogValue>),
    Table(Vec<Vec<String>>),
}

/// Response returned to the Lisp engine after dialog interaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DialogResponse {
    Cancelled,
    Submitted(HashMap<String, DialogValue>),
}
