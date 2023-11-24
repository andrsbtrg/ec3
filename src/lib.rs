pub mod error;
pub mod material_filter;
pub mod models;
pub mod utils;

use material_filter::MaterialFilter;
use models::Ec3Material;
use serde_json::Value;
use std::fmt::{self, Debug, Display, Formatter};

use crate::{error::ApiError, material_filter::convert};

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
}

pub enum Country {
    US,
    Germany,
    UK,
    None,
}

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

        println!("Querying materials...");

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

        let mut materials: Vec<Ec3Material> = Vec::new();

        json.as_array()
            .ok_or(ApiError::EmptyArray())?
            .iter()
            .for_each(|v| {
                let material: Ec3Material = serde_json::from_value(v.to_owned()).unwrap();

                materials.push(material);
            });
        let category = match &self.mf {
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
        }
    }
}
