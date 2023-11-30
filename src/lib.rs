pub mod error;
pub mod material_filter;
pub mod models;
pub mod utils;

use material_filter::MaterialFilter;
use models::Ec3Material;
use serde_json::Value;
use std::fmt::{self, Debug, Display, Formatter};

use crate::{
    error::ApiError,
    material_filter::convert,
    models::{Ec3Category, Node},
};

const BASE_PATH: &str = "https://buildingtransparency.org/api/";

/// Struct that can query the EC3 api for materials
pub struct Ec3api {
    api_key: String,
    endpoint: Endpoint,
    country: Country,
    mf: Option<MaterialFilter>,
    use_cache: bool,
}

pub enum Endpoint {
    Materials,
    Categories,
}

pub enum Country {
    US,
    Germany,
    UK,
    None,
}

pub enum Ec3Result {
    Materials(Vec<Ec3Material>),
    Categories(Node<Ec3Category>),
}
pub type APIResult = Result<Ec3Result, ApiError>;

/*
* Impl for EC3Api
*/
impl Ec3api {
    pub fn new(api_key: &str) -> Ec3api {
        Ec3api {
            api_key: api_key.to_string(),
            endpoint: Endpoint::Materials,
            country: Country::Germany,
            mf: None,
            use_cache: true,
        }
    }

    pub fn country(&mut self, country_code: Country) -> &mut Self {
        self.country = country_code;

        self
    }

    pub fn material_filter(&mut self, mf: MaterialFilter) -> &mut Self {
        self.mf = Some(mf);
        self
    }

    pub fn endpoint(&mut self, endpoint: Endpoint) -> &mut Self {
        self.endpoint = endpoint;

        self
    }
    fn prepare_url(&self) -> String {
        let jurisdiction = match self.country {
            Country::None => "".to_owned(),
            _ => format!("?jurisdiction={}", self.country),
        };
        let url = format!("{}{}{}", BASE_PATH, self.endpoint, jurisdiction);

        url
    }
    pub fn use_cache(&mut self, opt: bool) -> &mut Self {
        self.use_cache = opt;
        self
    }
    fn get_cached(&mut self) -> Option<Ec3Result> {
        match self.endpoint {
            Endpoint::Materials => {
                if let Some(mf) = &self.mf {
                    let category = mf.get_category();

                    if let Ok(ret) = utils::read_cache(&category) {
                        return Some(Ec3Result::Materials(ret));
                    } else {
                        println!("no cache found");
                        return None;
                    }
                } else {
                    eprintln!("Using cache requires specifying a MaterialFilter");
                    return None;
                }
            }
            Endpoint::Categories => None,
        }
    }
    pub fn fetch_all(&mut self) -> APIResult {
        if self.use_cache {
            if let Some(cached) = self.get_cached() {
                return Ok(cached);
            }
        }
        println!("Querying {}...", &self.endpoint);

        let path = self.prepare_url();

        let auth = format!("Bearer {}", self.api_key);

        let filter = if let Some(mf) = &self.mf {
            convert(mf)
        } else {
            String::new()
        };

        let response = ureq::get(&path)
            .set("Authorization", &auth)
            .query("mf", &filter)
            .call()
            .map_err(|_| ApiError::RequestError())?
            .into_string()?;

        let json: Value =
            serde_json::from_str(&response).map_err(|e| ApiError::DeserializationError(e))?;
        match self.endpoint {
            Endpoint::Materials => Ok(Ec3Result::Materials(get_materials(json, &self.mf)?)),

            Endpoint::Categories => Ok(Ec3Result::Categories(get_categories(json)?)),
        }
    }
    pub fn fetch(&mut self) -> Result<Vec<Ec3Material>, error::ApiError> {
        let category = match &self.mf {
            Some(mf) => mf.get_category(),
            None => "cache".to_string(),
        };

        if self.use_cache {
            if let Ok(ret) = utils::read_cache(&category) {
                return Ok(ret);
            } else {
                println!("no cache found");
            }
        }
        println!("Querying {}...", &self.endpoint);

        let path = self.prepare_url();

        let auth = format!("Bearer {}", self.api_key);

        let filter = if let Some(mf) = &self.mf {
            convert(mf)
        } else {
            String::new()
        };

        let response = ureq::get(&path)
            .set("Authorization", &auth)
            .query("mf", &filter)
            .call()
            .map_err(|_| ApiError::RequestError())?
            .into_string()?;

        let json: Value = serde_json::from_str(&response)?;

        Ok(get_materials(json, &self.mf)?)
    }
}

fn get_categories(json: Value) -> Result<Node<Ec3Category>, error::ApiError> {
    let mut root = Node::new();

    parse_tree(&json, &mut root);

    Ok(root)
}

fn parse_tree(json: &Value, parent: &mut Node<Ec3Category>) {
    if let Value::Object(root) = json {
        let subcs = root.get("subcategories").unwrap();
        parse_tree(subcs, parent);
        return;
    } else if let Value::Array(subc) = json {
        subc.iter().for_each(|v| {
            let name = v.get("name").unwrap().as_str().unwrap();

            let mut node = Node::name(name);
            parse_tree(v, &mut node);
            parent.add_children(node);
        })
    } else {
        println!("else")
    }
}

fn get_materials(
    json: Value,
    mf: &Option<MaterialFilter>,
) -> Result<Vec<Ec3Material>, error::ApiError> {
    let mut materials: Vec<Ec3Material> = Vec::new();

    json.as_array()
        .ok_or(ApiError::EmptyArray())?
        .iter()
        .for_each(|v| {
            let material: Ec3Material = serde_json::from_value(v.to_owned()).unwrap();

            materials.push(material);
        });
    let category = match &mf {
        Some(mf) => mf.get_category(),
        None => "cache".to_string(),
    };
    match serde_json::to_string_pretty(&materials) {
        Ok(json) => utils::write_cache(json, &category),
        Err(e) => {
            eprint!("Error: could not write cache: {e:?}");
        }
    };
    Ok(materials)
}

impl Debug for Ec3api {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ec3api")
            .field("api_key", &self.api_key)
            .finish()
    }
}
impl Display for Country {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match &self {
            Country::US => write!(f, "US"),
            Country::Germany => write!(f, "DE"),
            Country::UK => write!(f, "UK"),
            Country::None => write!(f, ""),
        }
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Endpoint::Materials => write!(f, "materials"),
            Endpoint::Categories => write!(f, "categories/root"),
        }
    }
}
