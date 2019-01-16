


![Megadex Logo](images/megadex.png)

##  Welcome to Megadex

Megadex is a simple tool that removes the boilerplate for indexing structs via multiple fields.
Presently it uses [rkv](https://github.com/mozilla/rkv) which uses [lmdb](https://github.com/danburkert/lmdb-rs). 
Though a purely in-mem version is planned. 

For every struct field you annotate, it will create additional accessor methods and indexes, including atomic insert/delete/get operations.  You can then fetch collections of your structs by matching a particular field.

When you derive(Megadex) you must supply at least an `#[id]` annotation on one of your fields that indicates that the field is the "primary key" of the struct.  Ids must be "string-ish" and they are expected to be unique in the db.
Upon doing so, these methods are created: 

```rust
fn Self::init(db) // required to establish the initial indices. 
fn Self::insert(db, key: &str, val: &Self) // a static method to insert an instance into the Db
fn Self::get(db, key: &str) // A static method to find a struct by its matching ID
fn Self::del(db, key: &str) // a static method to remove an instance from the Db

fn save(&self, db) // struct method to save/insert the current struct into the Db
fn erase(&self, db) // struct method to remove one's self from the DB
```

When you annotate a struct member with `#[indexed]` an index will be created in the underlying store and the following methods will be created: 

```rust
fn Self::find_by_<fieldname>(db, value: &str) // return an Iterator<Item=Self> of all instances whose field equals the supplied value
fn Self::id_by_<fieldname>(db, value: &str)  // return an Iterator<String> of the ids of all instances whose field matches the supplied value
```

## Example

```rust
use megadex_derive::Megadex;
use megadex::{ Db, MegadexDb, MegadexDbError };
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
```


## Future plans

* Better docs (obvs)
* Pure in-memory derivation (with no crate deps)
* Data-frame-esque features ala [Utah](https://github.com/kernelmachine/utah)
* Maybe `Apache Arrow` integration.

## Non plans

* Lots of relational features (just use sqlite) 
* Query Language beyond a simple DSL (see above) 


