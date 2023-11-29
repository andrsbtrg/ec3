use std::{path::Path, str::FromStr};

use crate::{
    error::ApiError,
    models::{Category, Ec3Material, Gwp, GwpUnits, Manufacturer},
};

pub fn write_cache(json: String, filename: &str) {
    let dir = Path::new(".cache");
    if !dir.exists() {
        println!("No cache folder, creating");
        std::fs::create_dir(dir).expect("Unable to create cache dir");
    }

    let output = Path::join(dir, Path::new(format!("{}.json", filename).as_str()));

    match std::fs::write(output, json) {
        Ok(_) => {
            println!("Results cached")
        }
        Err(e) => {
            println!("Could not write JSON file: {e:?}")
        }
    };
}
pub fn read_cache(category: &str) -> Result<Vec<Ec3Material>, crate::error::ApiError> {
    let dir = Path::new(".cache");

    let output = Path::join(dir, Path::new(format!("{}.json", category).as_str()));

    let contents = std::fs::read_to_string(output)?;

    let result: serde_json::Value = serde_json::from_str(&contents).unwrap();

    let mut out: Vec<Ec3Material> = Vec::new();

    result
        .as_array()
        .ok_or(ApiError::EmptyArray())?
        .iter()
        .for_each(|m| {
            let material = Ec3Material {
                name: m["name"].as_str().unwrap_or_default().to_string(),
                gwp: Gwp {
                    value: m["gwp"]["value"].as_f64().unwrap_or_default(),
                    unit: GwpUnits::from_str(m["gwp"]["unit"].as_str().unwrap_or_default())
                        .unwrap_or(GwpUnits::Unknown),
                },
                image: Some(m["image"].to_string()),
                manufacturer: Manufacturer {
                    name: m["manufacturer"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    country: Some(
                        m["manufacturer"]["country"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    ),
                },
                description: m["description"].as_str().unwrap_or("").to_string(),
                id: m["id"].as_str().unwrap_or("").to_string(),
                category: Category {
                    description: m["category"]["description"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    name: m["category"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    display_name: m["category"]["display_name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                    id: m["category"]["id"].as_str().unwrap_or("").to_string(),
                },
            };

            out.push(material);
        });

    Ok(out)
}
