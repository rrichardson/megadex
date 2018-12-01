mod error;

use crate::error::MegadexError;
use bincode;
use rkv::{Manager, MultiReader, Rkv, MultiStore, Value, MultiWriter, Iter as MDIter};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::convert::AsRef;
use std::fs;
use std::path::Path;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use tempfile::Builder;

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
    pub fn new<'p, P : Into<&'p Path>>(dir: P , fields: Vec<&str>) -> Result<Megadex<T>, MegadexError> {
        let mut writer = Manager::singleton()
            .write()
            .expect("Failed to get Manager Singleton writer");
        let env = writer.get_or_create(dir, Rkv::new)?;
        let store = env
            .write()
            .expect("failed to acquire env write lock")
            .open_or_create_multi(Some("_main_"))?;

        let mut md = 
            Megadex {
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

    pub fn get(&self, id: &str) -> Result<Option<T>, MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let reader = envlock.read_multi()?;
        if let Some(Value::Blob(blob)) = reader.get_first(self.main, id.as_bytes())? {
            bincode::deserialize(blob).map(Some).map_err(|e| e.into())
        } else {
            Ok(None)
        }
    }

    pub fn get_by_field(&self, name: &str, key: &[u8]) -> Result<Option<Vec<T>>, MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let reader = envlock.read_multi()?;
        let result : Result<Vec<T>, _> = 
            if let Some(ids) = self.get_ids_by_field(&reader, name, key)? {
                ids.map(|id| {
                    match reader.get_first(self.main, id.0)? {
                        Some(Value::Blob(o)) => bincode::deserialize(o).map_err(|e| e.into()),
                        None => Err(MegadexError::ValueError("Object not found for id".into())),
                        e => Err(MegadexError::InvalidType("Blob".into(), format!("{:?}", e))),
                    }
                }).collect()
            } else {
                return Ok(None);
            };
        result.map(Some)
    }

    pub fn get_ids_by_field<K: AsRef<[u8]>>(
        &self,
        reader: &MultiReader<K>,
        name: &str,
        key: K,
    ) -> Result<Option<MDIter>, MegadexError> {
        let idstore = self
            .indices
            .get(name)
            .ok_or_else(|| MegadexError::IndexUndefined(name.into()))?;
        reader.get(*idstore, key).map(Some).map_err(|e| e.into())
    }

    pub fn put(&self, id: &str, obj: &T, fields: &[(String, String)]) -> Result<(), MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let mut writer: MultiWriter<&str> = envlock.write_multi()?;
        self.put_id_txn(&mut writer, id, obj)?;
        for (field, key) in fields.iter() {
            self.put_field_txn(&mut writer, field, key.as_str(), id)?;
        }
        writer.commit().map_err(|e| e.into())
    }

    pub fn put_id_txn<'env, 's>(
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

    pub fn put_field_txn<'env, 's>(
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
    
    pub fn del(&self, id: &str, obj: &T, fields: &[(String, String)]) -> Result<(), MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let mut writer: MultiWriter<&str> = envlock.write_multi()?;
        let blob = bincode::serialize(obj).map_err(|e| -> MegadexError {  e.into()})?;
        writer
            .delete(self.main, id, &Value::Blob(&blob))
            .map_err(|e| -> MegadexError { e.into() })?;

        for (name, key) in fields {
            self.del_field_txn(&mut writer, name, key, id)?;
        }

        writer.commit().map_err(|e| e.into())
    }
    
    pub fn del_field_txn<K: AsRef<[u8]>>(
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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
