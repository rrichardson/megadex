#![feature(prelude_import)]
#![no_std]
#[prelude_import]
use std::prelude::v1::*;
#[macro_use]
extern crate std as std;

extern crate megadex;
extern crate megadex_derive;
extern crate serde_derive;
use bincode;
use megadex::{Db, MegadexDb, MegadexDbError};
use megadex_derive::Megadex;
use serde_derive::{Deserialize, Serialize};

pub struct Veggie {
    #[id]
    name: String,
    #[indexed]
    flavor: String,
    #[indexed]
    leaves: String,
    weight: f64,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::std::fmt::Debug for Veggie {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        match *self {
            Veggie {
                name: ref __self_0_0,
                flavor: ref __self_0_1,
                leaves: ref __self_0_2,
                weight: ref __self_0_3,
            } => {
                let mut debug_trait_builder = f.debug_struct("Veggie");
                let _ = debug_trait_builder.field("name", &&(*__self_0_0));
                let _ = debug_trait_builder.field("flavor", &&(*__self_0_1));
                let _ = debug_trait_builder.field("leaves", &&(*__self_0_2));
                let _ = debug_trait_builder.field("weight", &&(*__self_0_3));
                debug_trait_builder.finish()
            }
        }
    }
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_SERIALIZE_FOR_Veggie: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[allow(unused_macros)]
    #[automatically_derived]
    impl _serde::Serialize for Veggie {
        fn serialize<__S>(&self, __serializer: __S) -> _serde::export::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = match _serde::Serializer::serialize_struct(
                __serializer,
                "Veggie",
                0 + 1 + 1 + 1 + 1,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "name",
                &self.name,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "flavor",
                &self.flavor,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "leaves",
                &self.leaves,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "weight",
                &self.weight,
            ) {
                _serde::export::Ok(__val) => __val,
                _serde::export::Err(__err) => {
                    return _serde::export::Err(__err);
                }
            };
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_Veggie: () = {
    #[allow(unknown_lints)]
    #[allow(rust_2018_idioms)]
    extern crate serde as _serde;
    #[allow(unused_macros)]
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for Veggie {
        fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __field1,
                __field2,
                __field3,
                __ignore,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::export::Ok(__Field::__field0),
                        1u64 => _serde::export::Ok(__Field::__field1),
                        2u64 => _serde::export::Ok(__Field::__field2),
                        3u64 => _serde::export::Ok(__Field::__field3),
                        _ => _serde::export::Err(_serde::de::Error::invalid_value(
                            _serde::de::Unexpected::Unsigned(__value),
                            &"field index 0 <= i < 4",
                        )),
                    }
                }
                fn visit_str<__E>(self, __value: &str) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "name" => _serde::export::Ok(__Field::__field0),
                        "flavor" => _serde::export::Ok(__Field::__field1),
                        "leaves" => _serde::export::Ok(__Field::__field2),
                        "weight" => _serde::export::Ok(__Field::__field3),
                        _ => _serde::export::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::export::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"name" => _serde::export::Ok(__Field::__field0),
                        b"flavor" => _serde::export::Ok(__Field::__field1),
                        b"leaves" => _serde::export::Ok(__Field::__field2),
                        b"weight" => _serde::export::Ok(__Field::__field3),
                        _ => _serde::export::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(__deserializer: __D) -> _serde::export::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<'de> {
                marker: _serde::export::PhantomData<Veggie>,
                lifetime: _serde::export::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = Veggie;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::export::Formatter,
                ) -> _serde::export::fmt::Result {
                    _serde::export::Formatter::write_str(__formatter, "struct Veggie")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 =
                        match match _serde::de::SeqAccess::next_element::<String>(&mut __seq) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        } {
                            _serde::export::Some(__value) => __value,
                            _serde::export::None => {
                                return _serde::export::Err(_serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct Veggie with 4 elements",
                                ));
                            }
                        };
                    let __field1 =
                        match match _serde::de::SeqAccess::next_element::<String>(&mut __seq) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        } {
                            _serde::export::Some(__value) => __value,
                            _serde::export::None => {
                                return _serde::export::Err(_serde::de::Error::invalid_length(
                                    1usize,
                                    &"struct Veggie with 4 elements",
                                ));
                            }
                        };
                    let __field2 =
                        match match _serde::de::SeqAccess::next_element::<String>(&mut __seq) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        } {
                            _serde::export::Some(__value) => __value,
                            _serde::export::None => {
                                return _serde::export::Err(_serde::de::Error::invalid_length(
                                    2usize,
                                    &"struct Veggie with 4 elements",
                                ));
                            }
                        };
                    let __field3 =
                        match match _serde::de::SeqAccess::next_element::<f64>(&mut __seq) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        } {
                            _serde::export::Some(__value) => __value,
                            _serde::export::None => {
                                return _serde::export::Err(_serde::de::Error::invalid_length(
                                    3usize,
                                    &"struct Veggie with 4 elements",
                                ));
                            }
                        };
                    _serde::export::Ok(Veggie {
                        name: __field0,
                        flavor: __field1,
                        leaves: __field2,
                        weight: __field3,
                    })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::export::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::export::Option<String> = _serde::export::None;
                    let mut __field1: _serde::export::Option<String> = _serde::export::None;
                    let mut __field2: _serde::export::Option<String> = _serde::export::None;
                    let mut __field3: _serde::export::Option<f64> = _serde::export::None;
                    while let _serde::export::Some(__key) =
                        match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        }
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::export::Option::is_some(&__field0) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("name"),
                                    );
                                }
                                __field0 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<String>(&mut __map) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field1 => {
                                if _serde::export::Option::is_some(&__field1) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "flavor",
                                        ),
                                    );
                                }
                                __field1 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<String>(&mut __map) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field2 => {
                                if _serde::export::Option::is_some(&__field2) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "leaves",
                                        ),
                                    );
                                }
                                __field2 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<String>(&mut __map) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            __Field::__field3 => {
                                if _serde::export::Option::is_some(&__field3) {
                                    return _serde::export::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field(
                                            "weight",
                                        ),
                                    );
                                }
                                __field3 = _serde::export::Some(
                                    match _serde::de::MapAccess::next_value::<f64>(&mut __map) {
                                        _serde::export::Ok(__val) => __val,
                                        _serde::export::Err(__err) => {
                                            return _serde::export::Err(__err);
                                        }
                                    },
                                );
                            }
                            _ => {
                                let _ = match _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)
                                {
                                    _serde::export::Ok(__val) => __val,
                                    _serde::export::Err(__err) => {
                                        return _serde::export::Err(__err);
                                    }
                                };
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::export::Some(__field0) => __field0,
                        _serde::export::None => match _serde::private::de::missing_field("name") {
                            _serde::export::Ok(__val) => __val,
                            _serde::export::Err(__err) => {
                                return _serde::export::Err(__err);
                            }
                        },
                    };
                    let __field1 = match __field1 {
                        _serde::export::Some(__field1) => __field1,
                        _serde::export::None => {
                            match _serde::private::de::missing_field("flavor") {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            }
                        }
                    };
                    let __field2 = match __field2 {
                        _serde::export::Some(__field2) => __field2,
                        _serde::export::None => {
                            match _serde::private::de::missing_field("leaves") {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            }
                        }
                    };
                    let __field3 = match __field3 {
                        _serde::export::Some(__field3) => __field3,
                        _serde::export::None => {
                            match _serde::private::de::missing_field("weight") {
                                _serde::export::Ok(__val) => __val,
                                _serde::export::Err(__err) => {
                                    return _serde::export::Err(__err);
                                }
                            }
                        }
                    };
                    _serde::export::Ok(Veggie {
                        name: __field0,
                        flavor: __field1,
                        leaves: __field2,
                        weight: __field3,
                    })
                }
            }
            const FIELDS: &'static [&'static str] = &["name", "flavor", "leaves", "weight"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "Veggie",
                FIELDS,
                __Visitor {
                    marker: _serde::export::PhantomData::<Veggie>,
                    lifetime: _serde::export::PhantomData,
                },
            )
        }
    }
};
impl Veggie {
    pub fn find_by_flavor(
        md: &MegadexDb<Veggie>,
        field: &String,
    ) -> Result<Vec<Self>, MegadexDbError> {
        md.get_by_field("flavor", field)
    }
    pub fn id_by_flavor(
        md: &MegadexDb<Veggie>,
        key: &String,
    ) -> Result<Vec<String>, MegadexDbError> {
        let e = md.get_env();
        let envlock = e.read()?;
        let reader = envlock.read()?;
        md.get_ids_by_field(&reader, "flavor", key)
    }
    pub fn find_by_leaves(
        md: &MegadexDb<Veggie>,
        field: &String,
    ) -> Result<Vec<Self>, MegadexDbError> {
        md.get_by_field("leaves", field)
    }
    pub fn id_by_leaves(
        md: &MegadexDb<Veggie>,
        key: &String,
    ) -> Result<Vec<String>, MegadexDbError> {
        let e = md.get_env();
        let envlock = e.read()?;
        let reader = envlock.read()?;
        md.get_ids_by_field(&reader, "leaves", key)
    }
    pub fn init(db: Db) -> Result<MegadexDb<Veggie>, MegadexDbError> {
        MegadexDb::new(db, &["flavor", "leaves"])
    }
    pub fn save(&self, md: &mut MegadexDb<Veggie>) -> Result<(), MegadexDbError> {
        md.put(
            &self.name,
            self,
            &[
                (
                    "flavor",
                    &bincode::serialize(&self.flavor)
                        .map_err(|e| -> MegadexDbError { e.into() })?,
                ),
                (
                    "leaves",
                    &bincode::serialize(&self.leaves)
                        .map_err(|e| -> MegadexDbError { e.into() })?,
                ),
            ],
        )
    }
    pub fn erase(&self, md: &mut MegadexDb<Veggie>) -> Result<(), MegadexDbError> {
        md.del(
            &self.name,
            &[
                (
                    "flavor",
                    &bincode::serialize(&self.flavor)
                        .map_err(|e| -> MegadexDbError { e.into() })?,
                ),
                (
                    "leaves",
                    &bincode::serialize(&self.leaves)
                        .map_err(|e| -> MegadexDbError { e.into() })?,
                ),
            ],
        )
    }
    pub fn get(md: &MegadexDb<Veggie>, id: &String) -> Result<Option<Self>, MegadexDbError> {
        md.get(id)
    }
    pub fn del(md: &mut MegadexDb<Veggie>, id: &String) -> Result<(), MegadexDbError> {
        md.del(
            &id,
            &[
                (
                    "flavor",
                    &bincode::serialize(&val.flavor).map_err(|e| -> MegadexDbError { e.into() })?,
                ),
                (
                    "leaves",
                    &bincode::serialize(&val.leaves).map_err(|e| -> MegadexDbError { e.into() })?,
                ),
            ],
        )
    }
    pub fn insert(
        md: &mut MegadexDb<Veggie>,
        id: &String,
        val: &Veggie,
    ) -> Result<(), MegadexDbError> {
        md.put(
            &id,
            val,
            &[
                (
                    "flavor",
                    &bincode::serialize(&val.flavor).map_err(|e| -> MegadexDbError { e.into() })?,
                ),
                (
                    "leaves",
                    &bincode::serialize(&val.leaves).map_err(|e| -> MegadexDbError { e.into() })?,
                ),
            ],
        )
    }
}

fn check_veggies() {
    let db = Db::new_temp().unwrap();
    let mut md = Veggie::init(db).unwrap();

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
    Veggie::insert(&mut md, &"rhubarb".into(), &r).unwrap();

    let g1 = Veggie::get(&md, &"garlic".into()).unwrap().unwrap();
    let r1 = Veggie::get(&md, &"rhubarb".into()).unwrap().unwrap();

    let res = Veggie::find_by_flavor(&md, &"bold".into()).unwrap();

    let res = Veggie::id_by_leaves(&md, &"pointy".into()).unwrap();

    r1.erase(&mut md).unwrap();

    Veggie::del(&mut md, &"garlic".into()).unwrap();
}

fn main() {
    check_veggies();
}
