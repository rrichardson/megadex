
use megadex_derive::Megadex;
use megadex_rkv::{ Db, MegadexDb, MegadexDbError };
use serde_derive::{ Serialize, Deserialize };

#[derive(Debug, Serialize, Deserialize, Megadex)]

pub struct Veggie {
    #[id]
    name: String,
    #[indexed]
    flavor: String,
    #[indexed]
    leaves: String,
    weight: f64,
}

fn check_veggies() {
    let db = Db::new_temp().unwrap();
    let mut md =  Veggie::init(db).unwrap();

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

    r.save(&mut md).unwrap();
    Veggie::insert(&mut md, &"garlic".into(), &g).unwrap();

    let _g1 = Veggie::get(&md, &"garlic".into()).unwrap().unwrap();
    let r1 = Veggie::get(&md, &"rhubarb".into()).unwrap().unwrap();

    let _res = Veggie::find_by_flavor(&md, &"bold".into()).unwrap();

    let _res = Veggie::id_by_leaves(&md, &"pointy".into()).unwrap();

    r1.erase(&mut md).unwrap();

    Veggie::del(&mut md, &"garlic".into(), &g).unwrap();

}

fn main() {
    check_veggies();
}
