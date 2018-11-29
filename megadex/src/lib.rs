mod error;

use crate::error::MegadexError;
use bincode;
use rkv::{Manager, Reader, Rkv, Store, Value, Writer};
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::convert::AsRef;
use std::fs;
use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use tempfile::Builder;

pub struct Megadex<T> {
    env: Arc<RwLock<Rkv>>,
    main: Store,
    indices: HashMap<String, Store>,
    p: PhantomData<T>,
}

impl<T> Megadex<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new_temp() -> Result<Megadex<T>, MegadexError> {
        let root = Builder::new().prefix("megadex").tempdir()?;
        fs::create_dir_all(root.path())?;
        let mut writer = Manager::singleton()
            .write()
            .expect("Failed to get Manager Singleton writer");
        let env = writer.get_or_create(root.path(), Rkv::new)?;
        let store = env
            .write()
            .expect("failed to acquire env write lock")
            .open_or_create(Some("_main_"))?;
        Ok(Megadex {
            env,
            main: store,
            indices: HashMap::new(),
            p: PhantomData,
        })
    }

    pub fn new(rkv: Arc<RwLock<Rkv>>) -> Result<Megadex<T>, MegadexError> {
        let store = rkv
            .write()
            .expect("failed to acquire env write lock")
            .open_or_create(Some("_main_"))?;
        Ok(Megadex {
            env: rkv,
            main: store,
            indices: HashMap::new(),
            p: PhantomData,
        })
    }

    /// Create a new index to look up T
    pub fn index_new(&mut self, name: &str) -> Result<(), MegadexError> {
        let store = self
            .env
            .write()
                .expect("failed to acquire env write lock")
                .open_or_create(Some(name))?;
            self.indices.insert(name.into(), store);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<Option<T>, MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let reader = envlock.read()?;
        match reader.get(&self.main, id.as_bytes())? {
            Some(Value::Blob(o)) => bincode::deserialize(o).map_err(|e| e.into()),
            None => Err(MegadexError::ValueError("Object not found for id".into())),
            e => Err(MegadexError::InvalidType("Blob".into(), format!("{:?}", e))),
        }
    }

    pub fn get_by(&self, name: &str, key: &[u8]) -> Result<Option<T>, MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let reader = envlock.read()?;
        match self.get_id_by_txn(&reader, name, key)? {
            None => Ok(None),
            Some(id) => match reader.get(&self.main, id.as_bytes())? {
                Some(Value::Blob(o)) => bincode::deserialize(o).map_err(|e| e.into()),
                None => Err(MegadexError::ValueError("Object not found for id".into())),
                e => Err(MegadexError::InvalidType("Blob".into(), format!("{:?}", e))),
            },
        }
    }

    pub fn get_id_by(&self, name: &str, key: &[u8]) -> Result<Option<T>, MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let reader = envlock.read()?;
        match self.get_id_by_txn(&reader, name, key)? {
            Some(id) => match reader.get(&self.main, id.as_bytes())? {
                Some(Value::Blob(o)) => bincode::deserialize(o).map_err(|e| e.into()),
                None => Err(MegadexError::ValueError("Object not found for id".into())),
                e => Err(MegadexError::InvalidType("Blob".into(), format!("{:?}", e))),
            },
            None => Ok(None),
        }
    }

    pub fn get_id_by_txn<K: AsRef<[u8]>>(
        &self,
        reader: &Reader<K>,
        name: &str,
        key: K,
    ) -> Result<Option<String>, MegadexError> {
        let idstore = self
            .indices
            .get(name)
            .ok_or(MegadexError::IndexUndefined(name.into()))?;
        match reader.get(idstore, key)? {
            Some(Value::Str(id)) => Ok(Some(id.into())),
            None => Ok(None),
            e => Err(MegadexError::InvalidType("Str".into(), format!("{:?}", e))),
        }
    }

    pub fn put_id(&self, id: &str, obj: &T) -> Result<(), MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let mut writer: Writer<&[u8]> = envlock.write()?;
        self.put_id_txn(&mut writer, id.as_bytes(), obj)
    }

    pub fn put_id_txn<K: AsRef<[u8]>>(
        &self,
        writer: &mut Writer<K>,
        id: K,
        obj: &T,
    ) -> Result<(), MegadexError> {
        let blob = bincode::serialize(obj)?;
        writer
            .put(&self.main, id, &Value::Blob(&blob))
            .map_err(|e| e.into())
    }

    pub fn put_field_txn<K: AsRef<[u8]>>(
        &self,
        writer: &mut Writer<K>,
        field: K,
        id: &str,
    ) -> Result<(), MegadexError> {
        writer
            .put(&self.main, field, &Value::Str(id))
            .map_err(|e| e.into())
    }

    pub fn del_id_txn<K: AsRef<[u8]>>(
        &self,
        writer: &mut Writer<K>,
        id: K,
    ) -> Result<(), MegadexError> {
        writer.delete(&self.main, id).map_err(|e| e.into())
    }

    pub fn del(&self, id: &str, _indices: &Vec<(String, String)>) -> Result<(), MegadexError> {
        let envlock = self.env.read().expect("Failed to acquire read lock");
        let mut writer: Writer<&[u8]> = envlock.write()?;

        writer
            .delete(&self.main, id.as_bytes())
            .map_err(|e| -> MegadexError { e.into() })?;

        let _foreignkeys: Result<Vec<_>, MegadexError> =
            self.indices.iter().map(|(_name, _key)| Ok(())).collect();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
