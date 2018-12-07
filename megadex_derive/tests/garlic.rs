#[macro_use]
extern crate megadex_derive;

extern crate megadex;
use megadex::Megadex;

#[derive(Debug, Serialize, Deserialize, Megadex)]
pub struct Veggie {
    #[id]
    name: String,
    #[index]
    flavor: String,
    #[index]
    leaves: String,
    weight: f64,
}

#[test]
fn check_veggies() {
    let db = Db::new_temp().unwrap();
    let md: Veggie::init(db);

    let g = Veggie {
        name: "garlic".into(),
        flavor: "bold".into(),
        leaves: "pointy".into(),
        weight: 0.5,
    };
    
    let r = Veggie {
        name: "rhubarb".into(),
        flavor: "bold".into(),
        leaves: "broad".into(),
        weight: 2.5,
    };

    r.save(&md).unwrap();
    Veggie::insert(&md, "rhubarb", &r).unwrap();

    let g1 = Veggie::get(&md, "garlic").unwrap().unwrap();
    let r1 = Veggie::get(&md, "rhubarb").unwrap().unwrap();

    let res = Veggie::find_by_flavor(&md, "bold").unwrap();

    let res = Veggie::id_by_leaves(&md, "pointy").unwrap();

    r1.erase(&md).unwrap();

    Veggie::del(&md, "garlic", &g).unwrap();

}
