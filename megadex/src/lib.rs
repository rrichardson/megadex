mod error;

use bincode;
use rkv::{Iter as MDIter, Manager, MultiReader, MultiStore, MultiWriter, OwnedValue, Rkv, Value};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::convert::AsRef;
use std::fs;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tempfile::Builder;

#[cfg(test)]
use serde_derive;

use crate::error::MegadexError;

/// A specialized database that is persisted to the provided directory. This will store 
/// structs which implement `Serialize` and `DeserializeOwned`.  It will also index
/// those structs by any additional fields that you specify. 
///
/// This is a sparse and rather specialized API as it is intended to be used with 
/// the megadex_derive crate.
///
/// NOTE: If you plan on deleting this object later, DO NOT MUTATE it. 
/// Deletion from the db requires that the serialized bytes of T that are passed into
/// `del` must exactly match what is stored in the DB
pub struct Megadex<T> {
    env: Arc<RwLock<Rkv>>,
    main: MultiStore,
    indices: HashMap<String, MultiStore>,
    p: PhantomData<T>,
}

impl<T> Megadex<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Construct a new collection of indexes in a temp directory
    /// This will create The main struct store for T and
    /// the supporting secondary indexes to find the id for T
    pub fn new_temp(fields: Vec<&str>) -> Result<Megadex<T>, MegadexError> {
        let root = Builder::new().prefix("megadex").tempdir()?;
        fs::create_dir_all(root.path())?;
        let mut writer = Manager::singleton()
            .write()
            .expect("Failed to get Manager Singleton writer");
        let env = writer.get_or_create(root.path(), Rkv::new)?;
        let store = env
            .write()
            .expect("failed to acquire env write lock")
            .open_or_create_multi(Some("_main_"))?;

        let mut md = Megadex {
            env,
            main: store,
            indices: HashMap::new(),
            p: PhantomData,
        };
        md.insert_fields(fields)?;
        Ok(md)
    }

    /// Construct a new collection of indexes in the supplied directory.
    /// This will create The main struct store for T and
    /// the supporting secondary indexes to find the id for T
    pub fn new<'p, P: Into<&'p Path>>(
        dir: P,
        fields: Vec<&str>,
    ) -> Result<Megadex<T>, MegadexError> {
        let mut writer = Manager::singleton()
            .write()
            .expect("Failed to get Manager Singleton writer");
        let env = writer.get_or_create(dir, Rkv::new)?;
        let store = env
            .write()
            .expect("failed to acquire env write lock")
            .open_or_create_multi(Some("_main_"))?;

        let mut md = Megadex {
            env,
            main: store,
            indices: HashMap::new(),
            p: PhantomData,
        };
        md.insert_fields(fields)?;
        Ok(md)
    }

    fn insert_fields(&mut self, fields: Vec<&str>) -> Result<(), MegadexError> {
        for f in fields.into_iter() {
            let store = self
                .env
                .write()
                .expect("failed to acquire env write lock")
                .open_or_create_multi(Some(f))?;
            self.indices.insert(f.into(), store);
        }
        Ok(())
    }

    /// Retrieve T from the database at the given id. 
    /// Returns `None` if there is no value present for the id
    /// NOTE: If you plan on deleting this object later, DO NOT MUTATE it. 
    /// Deletion from the db requires that the serialized bytes of T that are passed into
    /// `del` must exactly match what is stored in the DB
    pub fn get(&self, id: &str) -> Result<Option<T>, MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let reader = envlock.read_multi()?;
        if let Some(OwnedValue::Blob(blob)) = reader.get_first(self.main, id.as_bytes())? {
            bincode::deserialize(&blob).map(Some).map_err(|e| e.into())
        } else {
            Ok(None)
        }
    }

    /// Retrieve all objects that are indexed by the provided field
    /// NOTE: If you plan on deleting this object later, DO NOT MUTATE it. 
    /// Deletion from the db requires that the serialized bytes of T that are passed into
    /// `del` must exactly match what is stored in the DB
    pub fn get_by_field(&self, name: &str, key: &str) -> Result<Vec<T>, MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let reader = envlock.read_multi()?;
        let res = self.get_ids_by_field(&reader, name, key)?;
        if res.is_some() {
            let ids = res.unwrap();
            ids.map(|(id, _)| {
                match reader.get_first(self.main, std::str::from_utf8(id).unwrap())? {
                    Some(OwnedValue::Blob(o)) => bincode::deserialize(&o).map_err(|e| e.into()),
                    None => Err(MegadexError::ValueError("Object not found for id".into())),
                    e => Err(MegadexError::InvalidType("Blob".into(), format!("{:?}", e))),
                }
            })
            .collect()
        } else {
            Ok(Vec::new())
        }
    }

    /// Retrieve all ids that are indexed by the provided field
    pub fn get_ids_by_field<'s>(
        &self,
        reader: &'s MultiReader<&'s str>,
        name: &str,
        key: &'s str,
    ) -> Result<Option<MDIter<'s>>, MegadexError> {
        let idstore = self
            .indices
            .get(name)
            .ok_or_else(|| MegadexError::IndexUndefined(name.into()))?;
        reader.get(*idstore, key).map(Some).map_err(|e| e.into())
    }

    /// Store an object of type T indexed by id
    pub fn put(&self, id: &str, obj: &T, fields: &[(String, String)]) -> Result<(), MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let mut writer: MultiWriter<&str> = envlock.write_multi()?;
        self.put_id_txn(&mut writer, id, obj)?;
        for (field, key) in fields.iter() {
            self.put_field_txn(&mut writer, field, key.as_str(), id)?;
        }
        writer.commit().map_err(|e| e.into())
    }

    fn put_id_txn<'env, 's>(
        &self,
        writer: &mut MultiWriter<'env, &'s str>,
        id: &'s str,
        obj: &T,
    ) -> Result<(), MegadexError> {
        let blob = bincode::serialize(obj)?;
        writer
            .put(self.main, id, &Value::Blob(&blob))
            .map_err(|e| e.into())
    }

    fn put_field_txn<'env, 's>(
        &self,
        writer: &mut MultiWriter<'env, &'s str>,
        field: &str,
        key: &'s str,
        id: &str,
    ) -> Result<(), MegadexError> {
        let idstore = self
            .indices
            .get(field)
            .ok_or_else(|| MegadexError::IndexUndefined(field.into()))?;
        writer
            .put(*idstore, key, &Value::Str(id))
            .map_err(|e| e.into())
    }

    /// Delete an object and all of its indexed fields. 
    /// Note that the obj, `T` must be in the exact state in which it was put into the DB
    /// for it to be successfully deleted. 
    pub fn del(&self, id: &str, obj: &T, fields: &[(String, String)]) -> Result<(), MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let mut writer: MultiWriter<&str> = envlock.write_multi()?;
        let blob = bincode::serialize(obj).map_err(|e| -> MegadexError { e.into() })?;
        writer
            .delete(self.main, id, &Value::Blob(&blob))
            .map_err(|e| -> MegadexError { e.into() })?;
        for (field, key) in fields {
            self.del_field_txn(&mut writer, field, key, id)?;
        }

        writer.commit().map_err(|e| e.into())
    }

    fn del_field_txn<K: AsRef<[u8]>>(
        &self,
        writer: &mut MultiWriter<K>,
        field: &str,
        key: K,
        id: &str,
    ) -> Result<(), MegadexError> {
        let idstore = self
            .indices
            .get(field)
            .ok_or_else(|| MegadexError::IndexUndefined(field.into()))?;
        writer
            .delete(*idstore, key, &Value::Str(id))
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct Weee {
        id: String,
        a: u32,
        b: String,
    }

    #[test]
    fn it_works() {
        let db: Megadex<Weee> = Megadex::new_temp(vec!["a", "b"]).unwrap();
        let w = Weee {
            id: "wat".into(),
            a: 42,
            b: "lalalala".into(),
        };

        db.put(&w.id, &w, &vec![("b".into(), w.b.clone())]).unwrap();
        let lala = db.get(&w.id).unwrap();
        assert_eq!(Some(w.clone()), lala);

        let ha = db.get_by_field("b", &w.b.clone()).unwrap();
        assert_eq!(ha, vec![w.clone()]);

        let res = db.get_by_field("c", &w.b.clone()).err().unwrap();
        assert_eq!(MegadexError::IndexUndefined("c".into()), res);

        db.del(&w.id, &w, &vec![("b".into(), w.b.clone())]).unwrap();

        let lala = db.get(&w.id).unwrap();
        assert_eq!(None, lala);

        let ha = db.get_by_field("b", &w.b.clone()).unwrap();
        assert_eq!(ha, vec![]);
    }
}
