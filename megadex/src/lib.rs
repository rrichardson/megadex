mod error;

use bincode;
use rkv::{
    Iter as MDIter,
    Manager,
    MultiReader,
    MultiStore,
    MultiWriter,
    OwnedValue,
    Rkv,
    Value,
};
use serde::{
    de::DeserializeOwned,
//    Deserialize,
    Serialize,
};
use std::collections::HashMap;
use std::convert::AsRef;
use std::fs;
use std::marker::PhantomData;
use std::path::Path;
use std::sync::{
    Arc,
    RwLock,
};
use tempfile::Builder;

#[cfg(test)]
use serde_derive;

pub use crate::error::MegadexDbError;

/// A specialized database environment that is persisted to the provided directory.
#[derive(Clone)]
pub struct Db {
    env: Arc<RwLock<Rkv>>,
}

impl Db {
    /// Construct a new collection of indexes in a temp directory
    /// This will create The main struct store for T and
    /// the supporting secondary indexes to find the id for T
    pub fn new_temp() -> Result<Db, MegadexDbError> {
        let root = Builder::new().prefix("megadex").tempdir()?;
        fs::create_dir_all(root.path())?;
        let mut writer = Manager::singleton().write().expect("Failed to get Manager Singleton writer");
        let env = writer.get_or_create(root.path(), Rkv::new)?;
        Ok(Db {
            env,
        })
    }

    /// Construct a new collection of indexes in the supplied directory,
    /// or if one already exists, it will use it.
    /// This will create The main struct store for T and
    /// the supporting secondary indexes to find the id for T
    pub fn new<'p, P: Into<&'p Path>>(dir: P) -> Result<Db, MegadexDbError> {
        let mut writer = Manager::singleton().write().expect("Failed to get Manager Singleton writer");
        let env = writer.get_or_create(dir, Rkv::new)?;
        Ok(Db {
            env,
        })
    }
}

/// A specialized database table that is persisted to the provided directory. This will store
/// structs which implement `Serialize` and `DeserializeOwned`.  It will also index
/// those structs by any additional fields that you specify.
///
/// This is a sparse and rather specialized API as it is intended to be used with
/// the megadex_derive crate.
///
/// NOTE: If you plan on deleting this object later, DO NOT MUTATE it.
/// Deletion from the db requires that the serialized bytes of T that are passed into
/// `del` must exactly match what is stored in the DB
pub struct MegadexDb<T> {
    env: Arc<RwLock<Rkv>>,
    main: MultiStore,
    indices: HashMap<String, MultiStore>,
    p: PhantomData<T>,
}

impl<T> MegadexDb<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(db: Db, fields: &[&str]) -> Result<MegadexDb<T>, MegadexDbError> {
        let env = db.env;
        let store = env.write().expect("failed to acquire env write lock").open_or_create_multi(Some("_main_"))?;

        let mut md = MegadexDb {
            env,
            main: store,
            indices: HashMap::new(),
            p: PhantomData,
        };
        md.insert_fields(fields)?;
        Ok(md)
    }

    fn insert_fields(&mut self, fields: &[&str]) -> Result<(), MegadexDbError> {
        for f in fields.into_iter() {
            let store =
                self.env.write().expect("failed to acquire env write lock").open_or_create_multi(Some(f.to_owned()))?;
            self.indices.insert((*f).into(), store);
        }
        Ok(())
    }

    /// Fetch a handle to the underlying LMDB environment
    pub fn get_env(&self) -> Arc<RwLock<Rkv>> {
        self.env.clone()
    }

    /// Retrieve T from the database at the given id.
    /// Returns `None` if there is no value present for the id
    /// NOTE: If you plan on deleting this object later, DO NOT MUTATE it.
    /// Deletion from the db requires that the serialized bytes of T that are passed into
    /// `del` must exactly match what is stored in the DB
    pub fn get<K: Serialize>(&self, id: &K) -> Result<Option<T>, MegadexDbError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let reader = envlock.read_multi()?;
        if let Some(OwnedValue::Blob(blob)) = reader.get_first(self.main, &bincode::serialize(id).map_err(MegadexDbError::from)?)? {
            bincode::deserialize(&blob).map(Some).map_err(|e| e.into())
        } else {
            Ok(None)
        }
    }

    /// Retrieve all objects that are indexed by the provided field
    /// NOTE: If you plan on deleting this object later, DO NOT MUTATE it.
    /// Deletion from the db requires that the serialized bytes of T that are passed into
    /// `del` must exactly match what is stored in the DB
    pub fn get_by_field<K: Serialize>(&self, name: &str, key: &K) -> Result<Vec<T>, MegadexDbError> {
        let keybytes = bincode::serialize(key).map_err(MegadexDbError::from)?;
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let reader = envlock.read_multi()?;
        let res = self.get_ids_by_field_raw(&reader, name, &keybytes)?;
        if res.is_some() {
            let ids = res.unwrap();
            ids.map(|(id, _)| match reader.get_first(self.main, id)? {
                Some(OwnedValue::Blob(o)) => bincode::deserialize(&o).map_err(|e| e.into()),
                None => Err(MegadexDbError::ValueError("Object not found for id".into())),
                e => Err(MegadexDbError::InvalidType("Blob".into(), format!("{:?}", e))),
            })
            .collect()
        } else {
            Ok(Vec::new())
        }
    }
    
    /// Retrieve the exact type of ids that are indexed by the provided field
    /// XXX Note that this will basically swallow deserialization and mismatchd type errors by
    /// simpling excluding the result from the vector if it fails
    pub fn get_ids_by_field<'s, K: Serialize, I: DeserializeOwned>(
        &self,
        reader: &'s MultiReader<&'s [u8]>,
        name: &str,
        key: &'s K,
    ) -> Result<Vec<I>, MegadexDbError> {
        let unpack = |obj : Result<Option<Value>, MegadexDbError>| -> Option<I> {
            match obj {
                Ok(Some(Value::Blob(bytes))) => {
                    bincode::deserialize(bytes).map_err(|e : bincode::Error| -> MegadexDbError {  e.into() }).ok()
                }, 
                Ok(Some(_)) => None,
                Ok(None) => None,
                Err(_) => None,
            }
        };

        match self.get_ids_by_field_raw(reader, name, &bincode::serialize(key).map_err(MegadexDbError::from)?)? {
            None => Ok(Vec::new()),
            Some(iter) => Ok(iter.map(|(_, v)| unpack(v.map_err(|e| e.into()))).flatten().collect::<Vec<I>>()),
        }
    }

    /// Retrieve an iterator for the raw bytes of ids that are indexed by the provided field
    pub fn get_ids_by_field_raw<'s>(
        &self,
        reader: &'s MultiReader<&'s [u8]>,
        name: &str,
        key: &'s [u8],
    ) -> Result<Option<MDIter<'s>>, MegadexDbError> {
        let idstore = self.indices.get(name).ok_or_else(|| MegadexDbError::IndexUndefined(name.into()))?;
        reader.get(*idstore, key).map(Some).map_err(|e| e.into()) 
    }

    /// Store an object of type T indexed by id
    pub fn put<K: Serialize>(&self, id: &K, obj: &T, fields: &[(&str, &[u8])]) -> Result<(), MegadexDbError> {
        let keybytes = bincode::serialize(id).map_err(MegadexDbError::from)?;
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let mut writer: MultiWriter<&[u8]> = envlock.write_multi()?;
        self.put_id_txn(&mut writer, &keybytes, obj)?;
        for (field, key) in fields.iter() {
            self.put_field_txn(&mut writer, field, key, &keybytes)?;
        }
        writer.commit().map_err(|e| e.into())
    }

    fn put_id_txn<'env, 's>(
        &self,
        writer: &mut MultiWriter<'env, &'s [u8]>,
        id: &'s [u8],
        obj: &T,
    ) -> Result<(), MegadexDbError> {
        let blob = bincode::serialize(obj)?;
        writer.put(self.main, id, &Value::Blob(&blob)).map_err(|e| e.into())
    }

    fn put_field_txn<'env, 's>(
        &self,
        writer: &mut MultiWriter<'env, &'s [u8]>,
        field: &str,
        key: &'s [u8],
        id: &[u8],
    ) -> Result<(), MegadexDbError> {
        let idstore = self.indices.get(field).ok_or_else(|| MegadexDbError::IndexUndefined(field.into()))?;
        writer.put(*idstore, key, &Value::Blob(id)).map_err(|e| e.into())
    }

    /// Delete an object and all of its indexed fields.
    /// Note that the obj, `T` must be in the exact state in which it was put into the DB
    /// for it to be successfully deleted.
    pub fn del<K: Serialize>(&self, id: &K, obj: &T, fields: &[(&str, &[u8])]) -> Result<(), MegadexDbError> {
        let keybytes = bincode::serialize(id).map_err(MegadexDbError::from)?;
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let mut writer: MultiWriter<&[u8]> = envlock.write_multi()?;
        let blob = bincode::serialize(obj).map_err(|e| -> MegadexDbError { e.into() })?;
        writer.delete(self.main, &keybytes, &Value::Blob(&blob)).map_err(|e| -> MegadexDbError { e.into() })?;
        for (field, key) in fields {
            self.del_field_txn(&mut writer, field, key, &keybytes)?;
        }

        writer.commit().map_err(|e| e.into())
    }

    fn del_field_txn<K: AsRef<[u8]>>(
        &self,
        writer: &mut MultiWriter<K>,
        field: &str,
        key: K,
        id: &[u8],
    ) -> Result<(), MegadexDbError> {
        let idstore = self.indices.get(field).ok_or_else(|| MegadexDbError::IndexUndefined(field.into()))?;
        writer.delete(*idstore, key, &Value::Blob(id)).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_derive::{
        Deserialize,
        Serialize,
    };

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct Weee {
        id: String,
        a: u32,
        b: String,
    }

    #[test]
    fn it_works() {
        let db = Db::new_temp().unwrap();
        let md: MegadexDb<Weee> = MegadexDb::new(db, &["a", "b"][..]).unwrap();
        let w = Weee {
            id: "wat".into(),
            a: 42,
            b: "lalalala".into(),
        };

        md.put(&w.id, &w, &vec![("b", w.b.as_bytes())]).unwrap();
        let lala = md.get(&w.id).unwrap();
        assert_eq!(Some(w.clone()), lala);

        let ha = md.get_by_field("b", &w.b).unwrap();
        assert_eq!(ha, vec![w.clone()]);

        let res = md.get_by_field("c", &w.b).err().unwrap();
        assert_eq!(MegadexDbError::IndexUndefined("c".into()), res);

        md.del(&w.id, &w, &[("b".into(), w.b.as_bytes())]).unwrap();

        let lala = md.get(&w.id).unwrap();
        assert_eq!(None, lala);

        let ha = md.get_by_field("b", &w.b.as_bytes()).unwrap();
        assert_eq!(ha, vec![]);
    }
}
