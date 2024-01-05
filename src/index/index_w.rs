use std::sync::{Arc, Mutex};

use flurry::HashMap;
use once_cell::sync::Lazy;
use tantivy::{Index, IndexWriter, Opstamp, Document};


pub struct IndexW {
    pub path: String,
    pub index: Index,
    pub writer: Mutex<Option<IndexWriter>>,
}


impl IndexW {
    // wrapper for IndexWriter.commit
    pub fn commit(&self) -> Result<Opstamp, String> {
        match self.writer.lock() {
            Ok(mut writer) => {
                if let Some(writer) = writer.as_mut() {
                    writer.commit().map_err(|e| e.to_string())
                } else {
                    Err("IndexWriter is not available".to_string())
                }
            },
            Err(e) => Err(format!("Lock error: {}", e)),
        }
    }

    // wrapper for IndexWriter.add_document
    pub fn add_document(&self, document: Document) -> Result<Opstamp, String> {
        match self.writer.lock() {
            Ok(mut writer) => {
                if let Some(writer) = writer.as_mut() {
                    writer.add_document(document).map_err(|e| e.to_string())
                } else {
                    Err("IndexWriter is not available".to_string())
                }
            },
            Err(e) => Err(format!("Lock error: {}", e)),
        }
    }

    // wrapper for IndexWriter.wait_merging_threads.
    pub fn wait_merging_threads(&self) -> Result<(), String> {
        // use Interior Mutability
        match self.writer.lock() {
            Ok(mut writer) => {
                
                if let Some(writer) = writer.take() {
                    let _ = writer.wait_merging_threads();
                };
                Ok(())
            },
            Err(e) => {
                Err(format!("Failed to acquire lock in drop: {}", e.to_string()))
            },
        }
    }
}


impl Drop for IndexW {
    fn drop(&mut self) {
        println!("IndexW has been dropped.");
    }
}



// cache store IndexW for thread safe
static INDEXW_CACHE: Lazy<Arc<HashMap<String, Arc<IndexW>>>> = Lazy::new(|| Arc::new(HashMap::new()));


pub fn get_index_w(key: String) -> Result<Arc<IndexW>, String> {
    let pinned = INDEXW_CACHE.pin();
    match pinned.get(&key) {
        Some(result) => Ok(result.clone()),
        None => Err(format!("Index doesn't exist with given key: [{}]", key)),
    }
}

pub fn set_index_w(key: String, value: Arc<IndexW>) -> Result<(), String> {
    let pinned = INDEXW_CACHE.pin();
    if pinned.contains_key(&key) {
        pinned.insert(key.clone(), value.clone());
        Err(format!(
            "Index already exists with given key: [{}], it has been overwritten.",
            key
        ))
    } else {
        pinned.insert(key, value.clone());
        Ok(())
    }
}
pub fn remove_index_w(key: String) -> Result<(), String> {
    let pinned = INDEXW_CACHE.pin();
    if pinned.contains_key(&key) {
        pinned.remove(&key);
        Ok(())
    } else {
        Err(format!(
            "Index doesn't exist, can't remove it with given key: [{}]",
            key
        ))
    }
}