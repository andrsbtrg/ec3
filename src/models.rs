use crate::error::ApiError;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Ec3Category {
    pub name: String,
}
impl Default for Ec3Category {
    fn default() -> Self {
        Self {
            name: "ConstructionMaterials".to_string(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct Node<T> {
    pub children: Option<Vec<Self>>,
    pub value: T,
}

impl Node<Ec3Category> {
    pub fn new() -> Self {
        Node {
            children: Some(Vec::new()),
            value: Ec3Category::default(),
        }
    }
    pub fn name(name: &str) -> Self {
        Node {
            children: Some(Vec::new()),
            value: Ec3Category {
                name: name.to_string(),
            },
        }
    }

    pub fn add_children(&mut self, node: Node<Ec3Category>) {
        match &mut self.children {
            Some(c) => c.push(node),
            None => todo!(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ec3Material {
    pub name: String,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub gwp: Gwp,
    #[serde(default)]
    pub image: Option<String>,
    pub manufacturer: Manufacturer,
    pub description: String,
    pub category: Category,
    pub id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manufacturer {
    pub name: String,
    pub country: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Category {
    pub description: String,
    pub name: String,
    pub display_name: String,
    pub id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Gwp {
    pub value: f64,
    pub unit: GwpUnits,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GwpUnits {
    KgCO2e,
    Unknown,
}

impl FromStr for GwpUnits {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "KgCO2e" => Ok(Self::KgCO2e),
            "kgCO2e" => Ok(Self::KgCO2e),
            "kgCo2e" => Ok(Self::KgCO2e),
            _ => Ok(Self::Unknown),
        }
    }
}
// You can use this deserializer for any type that implements FromStr
// and the FromStr::Err implements Display
fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,                // Required for S::from_str...
    S::Err: std::fmt::Display, // Required for .map_err(de::Error::custom)
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(de::Error::custom)
}

impl Gwp {
    pub fn as_str(&self) -> String {
        format!("{} {:?}", self.value, self.unit)
    }
}

impl Default for Gwp {
    fn default() -> Self {
        Self {
            value: 0 as f64,
            unit: GwpUnits::KgCO2e,
        }
    }
}

impl FromStr for Gwp {
    type Err = crate::error::ApiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x, y) = s.split_once(' ').ok_or(ApiError::GwpError)?;
        let value = x.parse::<f64>().map_err(|_| ApiError::GwpError)?;
        let unit = y.parse::<GwpUnits>().map_err(|_| ApiError::GwpError)?;
        Ok(Gwp { value, unit })
    }
}
