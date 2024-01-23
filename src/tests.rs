use crate::error;
use crate::material_filter::convert;
use crate::material_filter::MaterialFilter;
use crate::Ec3Result;
use crate::{Ec3api, Endpoint};
use dotenv::dotenv;

type Result<T> = std::result::Result<T, error::ApiError>;

#[test]
fn fetch_from_category() -> Result<()> {
    dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("API_KEY env variable not set.");
    let mut mf = MaterialFilter::of_category("Wood");
    mf.add_filter("jurisdiction", "in", vec!["150"]);
    mf.add_filter("epd_types", "in", vec!["Product EPDs", "Industry EPDs"]);

    let materials = Ec3api::new(&api_key)
        .endpoint(Endpoint::Materials)
        .use_cache(false)
        .material_filter(mf)
        .fetch()?;
    println!("{:?}", materials.first());
    assert!(materials.len() > 0, "Fetch returned no results");
    Ok(())
}

#[test]
fn fetch_categories() {
    dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("API_KEY env variable not set.");

    let api_result = Ec3api::new(&api_key)
        .endpoint(Endpoint::Categories)
        .fetch_all()
        .unwrap();
    if let Ec3Result::Categories(categories) = api_result {
        // let txt = format!("{:?}", &categories);
        // let _ = std::fs::write("./node.txt", txt);
        let children = categories.children.expect("Node yielded no children");
        let concrete = children
            .iter()
            .find(|x| x.value.name == "Concrete")
            .expect("Expected 'Concrete' node");
        assert!(concrete.value.declared_unit.unit == crate::models::Unit::M3);
    };
}

#[test]
fn test_material_filter() {
    let mut mf = MaterialFilter::of_category("Concrete");
    mf.add_filter("jurisdiction", "in", vec!["150"]);
    mf.add_filter("epd_types", "in", vec!["Product EPDs", "Industry EPDs"]);

    let converted = r#"!EC3 search("Concrete") WHERE
 jurisdiction: IN("150") AND
 epd_types: IN("Product EPDs", "Industry EPDs")
!pragma eMF("2.0/1"), lcia("EF 3.0")"#;

    assert_eq!(convert(&mf).as_str(), converted)
}
