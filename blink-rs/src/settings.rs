use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NumberRange {
    pub min: f64,
    pub max: f64,
}

impl NumberRange {
    pub fn new(min: f64, max: Option<f64>) -> Self {
        NumberRange {
            min,
            max: max.unwrap_or(min),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Case {
    Camel,
    Snake,
    Pascal,
}

#[derive(Debug, Clone)]
pub struct Cases {
    pub fire: String,
    pub fire_all: String,
    pub fire_list: String,
    pub fire_except: String,
    pub on: String,
    pub invoke: String,
    pub step_replication: String,
    pub next: String,
    pub iter: String,
    pub read: String,
    pub write: String,
}

#[derive(Debug, Clone)]
pub struct Primitive {
    pub bounds: Option<NumberRange>,
    pub integer: Option<bool>,
    pub component: bool,
    pub allows_range: bool,
    pub allows_optional: bool,
    pub allowed_components: usize,
}

static CASE_OPTIONS: &[(&str, [&str; 3])] = &[
    ("On", ["On", "on", "on"]),
    ("Invoke", ["Invoke", "invoke", "invoke"]),
    ("Fire", ["Fire", "fire", "fire"]),
    ("FireAll", ["FireAll", "fireAll", "fire_all"]),
    ("FireList", ["FireList", "fireList", "fire_list"]),
    ("FireExcept", ["FireExcept", "fireExcept", "fire_except"]),
    (
        "StepReplication",
        ["StepReplication", "stepReplication", "step_replication"],
    ),
    ("Next", ["Next", "next", "next"]),
    ("Iter", ["Iter", "iter", "iter"]),
    ("Read", ["Read", "read", "read"]),
    ("Write", ["Write", "write", "write"]),
];

pub fn get_casing(case: &Case) -> Cases {
    let index = match case {
        Case::Pascal => 0,
        Case::Camel => 1,
        Case::Snake => 2,
    };

    Cases {
        on: CASE_OPTIONS.iter().find(|(k, _)| *k == "On").unwrap().1[index].to_string(),
        invoke: CASE_OPTIONS.iter().find(|(k, _)| *k == "Invoke").unwrap().1[index].to_string(),
        fire: CASE_OPTIONS.iter().find(|(k, _)| *k == "Fire").unwrap().1[index].to_string(),
        fire_all: CASE_OPTIONS
            .iter()
            .find(|(k, _)| *k == "FireAll")
            .unwrap()
            .1[index]
            .to_string(),
        fire_list: CASE_OPTIONS
            .iter()
            .find(|(k, _)| *k == "FireList")
            .unwrap()
            .1[index]
            .to_string(),
        fire_except: CASE_OPTIONS
            .iter()
            .find(|(k, _)| *k == "FireExcept")
            .unwrap()
            .1[index]
            .to_string(),
        step_replication: CASE_OPTIONS
            .iter()
            .find(|(k, _)| *k == "StepReplication")
            .unwrap()
            .1[index]
            .to_string(),
        next: CASE_OPTIONS.iter().find(|(k, _)| *k == "Next").unwrap().1[index].to_string(),
        iter: CASE_OPTIONS.iter().find(|(k, _)| *k == "Iter").unwrap().1[index].to_string(),
        read: CASE_OPTIONS.iter().find(|(k, _)| *k == "Read").unwrap().1[index].to_string(),
        write: CASE_OPTIONS.iter().find(|(k, _)| *k == "Write").unwrap().1[index].to_string(),
    }
}

pub fn get_keywords() -> HashMap<String, bool> {
    let mut keywords = HashMap::new();
    for &kw in &[
        "map", "set", "type", "enum", "struct", "event", "function", "scope", "option", "export",
    ] {
        keywords.insert(kw.to_string(), true);
    }
    keywords
}

pub fn get_primitives() -> HashMap<String, Primitive> {
    let mut primitives = HashMap::new();

    primitives.insert(
        "u8".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(0.0, Some(255.0))),
            integer: Some(true),
            component: true,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "u16".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(0.0, Some(65535.0))),
            integer: Some(true),
            component: true,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "u32".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(0.0, Some(4294967295.0))),
            integer: Some(true),
            component: true,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "i8".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(-128.0, Some(127.0))),
            integer: Some(true),
            component: true,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "i16".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(-32768.0, Some(32767.0))),
            integer: Some(true),
            component: true,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "i32".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(-2147483648.0, Some(2147483647.0))),
            integer: Some(true),
            component: true,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "f16".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(-65504.0, Some(65504.0))),
            integer: None,
            component: true,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "f32".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(-16777216.0, Some(16777216.0))),
            integer: None,
            component: true,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "f64".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(
                -9007199254740992.0,
                Some(9007199254740992.0),
            )),
            integer: None,
            component: true,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "boolean".to_string(),
        Primitive {
            bounds: None,
            integer: None,
            component: false,
            allows_range: false,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "string".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(0.0, Some(4294967295.0))),
            integer: Some(true),
            component: false,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "vector".to_string(),
        Primitive {
            bounds: None,
            integer: None,
            component: false,
            allows_range: true,
            allows_optional: true,
            allowed_components: 1,
        },
    );
    primitives.insert(
        "buffer".to_string(),
        Primitive {
            bounds: Some(NumberRange::new(0.0, Some(4294967295.0))),
            integer: Some(true),
            component: false,
            allows_range: true,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "Color3".to_string(),
        Primitive {
            bounds: None,
            integer: None,
            component: false,
            allows_range: false,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "CFrame".to_string(),
        Primitive {
            bounds: None,
            integer: None,
            component: false,
            allows_range: false,
            allows_optional: true,
            allowed_components: 2,
        },
    );
    primitives.insert(
        "Instance".to_string(),
        Primitive {
            bounds: None,
            integer: None,
            component: false,
            allows_range: false,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "BrickColor".to_string(),
        Primitive {
            bounds: None,
            integer: None,
            component: false,
            allows_range: false,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "unknown".to_string(),
        Primitive {
            bounds: None,
            integer: None,
            component: false,
            allows_range: false,
            allows_optional: false,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "DateTime".to_string(),
        Primitive {
            bounds: None,
            integer: None,
            component: false,
            allows_range: false,
            allows_optional: true,
            allowed_components: 0,
        },
    );
    primitives.insert(
        "DateTimeMillis".to_string(),
        Primitive {
            bounds: None,
            integer: None,
            component: false,
            allows_range: false,
            allows_optional: true,
            allowed_components: 0,
        },
    );

    primitives
}
