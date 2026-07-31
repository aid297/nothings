use crate::any_slices::app::AnySlice;
use std::collections::HashMap;
use std::io::Error;

pub struct AnyMap<K, V> {
    keys: AnySlice<K>,
    values: AnySlice<V>,
}

impl<K, V> AnyMap<K, V> {
    pub fn new() -> Self {
        AnyMap {
            keys: AnySlice::new(vec![]),
            values: AnySlice::new(vec![]),
        }
    }
    
    pub fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item=(K, V)>,
    {
        let mut keys: AnySlice<K> = AnySlice::new(vec![]);
        let mut values: AnySlice<V> = AnySlice::new(vec![]);
        
        for (k, v) in iter {
            keys.push(k);
            values.push(v);
        }
        
        AnyMap { keys, values }
    }
    
    pub fn from_hashmap(hash_map: HashMap<K, V>) -> Self {
        let mut keys: AnySlice<K> = AnySlice::new(vec![]);
        let mut values: AnySlice<V> = AnySlice::new(vec![]);
        
        for (k, v) in hash_map.into_iter() {
            keys.push(k);
            values.push(v);
        }
        
        AnyMap { keys, values }
    }
    
    pub fn set_data(&mut self, has_map: HashMap<K, V>) -> &Self {
        let mut keys: AnySlice<K> = AnySlice::new(vec![]);
        let mut values: AnySlice<V> = AnySlice::new(vec![]);
        
        for (k, v) in has_map.into_iter() {
            keys.push(k);
            values.push(v);
        }
        
        self.keys = keys;
        self.values = values;
        self
    }
    
    pub fn push_datum(&mut self, key: K, value: V) -> &Self {
        self.keys.push(key);
        self.values.push(value);
        self
    }
    
    pub fn set_value_by_index(&mut self, index: usize, value: V) -> &Self
    where
        V: Clone,
    {
        self.values.set_value(index, value);
        
        self
    }
    
    pub fn set_value_by_key(&mut self, key: K, value: V) -> Result<&Self, Error>
    where
        K: PartialEq,
        V: Clone,
    {
        let index = self.keys.get_index_by_value(&key);
        match index {
            None => Err(Error::new(std::io::ErrorKind::NotFound, "未找到 key")),
            Some(idx) => {
                self.values.set_value(idx, value);
                Ok(self)
            }
        }
    }
    
    pub fn get_value_by_key(&self, key: &K) -> Result<&V, Error>
    where
        K: PartialEq,
    {
        let idx = self.keys.get_index_by_value(key);
        match idx {
            None => Err(Error::new(std::io::ErrorKind::NotFound, "未找到 key")),
            Some(idx) => {
                let value = self.values.get_value_by_index(idx);
                match value {
                    None => Err(Error::new(std::io::ErrorKind::NotFound, "未找到 value")),
                    Some(x) => Ok(x),
                }
            }
        }
    }
    
    pub fn to_hashmap(&self) -> HashMap<K, V>
    where
        K: Clone + Eq + std::hash::Hash,
        V: Clone,
    {
        let mut hashmap = HashMap::new();
        
        for idx in self.keys.get_indexes() {
            if let (Some(k), Some(v)) = (self.keys.get_value_by_index(idx), self.values.get_value_by_index(idx)) {
                hashmap.insert(k.clone(), v.clone());
            }
        }
        
        hashmap
    }
    
    pub fn copy(&self) -> Self
    where
        K: Clone,
        V: Clone,
    {
        AnyMap {
            keys: self.keys.copy(),
            values: self.values.copy(),
        }
    }
    
    pub fn has(&self, key: &K) -> bool
    where
        K: PartialEq + Clone,
    {
        self.keys.has(&vec![key.clone()])
    }
    
    pub fn filter(&mut self, f: impl Fn(&K, &V) -> bool) -> &Self
    where
        K: Clone,
        V: Clone,
    {
        let mut wait_to_remove_indexes: Vec<usize> = vec![];
        
        for idx in self.keys.get_indexes() {
            let key = self.keys.get_value_by_index(idx).unwrap();
            let value = self.values.get_value_by_index(idx).unwrap();
            
            if !f(key, value) {
                wait_to_remove_indexes.push(idx);
            }
        }
        
        self.keys
            .remove_by_indexes(&wait_to_remove_indexes.to_vec());
        self.values
            .remove_by_indexes(&wait_to_remove_indexes.to_vec());
        
        self
    }
    
    pub fn remove_empty(&mut self) -> &Self
    where
        K: Clone,
        V: PartialEq + Default + Clone,
    {
        let wait_to_remove_indexes: Vec<usize> = self
            .values
            .get_indexes()
            .into_iter()
            .filter(|idx| self.values.get_value_by_index(*idx).unwrap() == &V::default())
            .collect();
        
        self.keys
            .remove_by_indexes(&wait_to_remove_indexes.to_vec());
        self.values
            .remove_by_indexes(&wait_to_remove_indexes.to_vec());
        
        self
    }
    
    pub fn remove_by_key(&mut self, key: &K) -> &Self
    where
        K: PartialEq,
    {
        let idx = self.keys.get_index_by_value(key);
        match idx {
            None => self,
            Some(idx) => {
                self.keys.remove_by_index(&idx);
                self.values.remove_by_index(&idx);
                self
            }
        }
    }
    
    pub fn remove_by_index(&mut self, index: usize) -> &Self {
        if index >= self.keys.len() || index >= self.values.len() {
            return self;
        }
        self.keys.remove_by_index(&index);
        self.values.remove_by_index(&index);
        self
    }
    
    pub fn remove_by_keys(&mut self, keys: &Vec<K>) -> &Self
    where
        K: PartialEq,
    {
        let indexes = self.get_indexes_by_keys(keys);
        self.keys.remove_by_indexes(&indexes);
        self.values.remove_by_indexes(&indexes);
        
        self
    }
    
    pub fn remove_by_values(&mut self, values: &Vec<V>) -> &Self
    where
        V: PartialEq,
    {
        let indexes = self.get_indexes_by_values(values);
        self.keys.remove_by_indexes(&indexes);
        self.values.remove_by_indexes(&indexes);
        
        self
    }
    
    pub fn get_index_by_key(&self, key: &K) -> Option<usize>
    where
        K: PartialEq,
    {
        self.keys.get_index_by_value(key)
    }
    
    pub fn get_indexes_by_keys(&self, keys: &[K]) -> Vec<usize>
    where
        K: PartialEq,
    {
        let mut indexes: Vec<usize> = keys
            .iter()
            .filter_map(|k| self.keys.get_index_by_value(k))
            .collect();
        indexes.sort_unstable();
        indexes.dedup();
        indexes
    }
    
    pub fn get_indexes_by_values(&self, values: &[V]) -> Vec<usize>
    where
        V: PartialEq,
    {
        let mut indexes: Vec<usize> = values
            .iter()
            .filter_map(|v| self.values.get_index_by_value(v))
            .collect();
        indexes.sort_unstable();
        indexes.dedup();
        indexes
    }
    
    pub fn in_key(&self, key: &K) -> bool
    where
        K: PartialEq + Clone,
    {
        self.keys.has(&vec![key.clone()])
    }
    
    pub fn in_keys(&self, keys: &Vec<K>) -> bool
    where
        K: PartialEq,
    {
        keys.iter().all(|key| self.keys.to_vec().contains(key))
    }
    
    pub fn not_in_key(&self, key: &K) -> bool
    where
        K: PartialEq + Clone,
    {
        self.keys.not_has(&vec![key.clone()])
    }
    
    pub fn not_in_keys(&self, keys: &Vec<K>) -> bool
    where
        K: PartialEq,
    {
        self.keys.not_has(keys)
    }
    
    pub fn in_value(&self, value: &V) -> bool
    where
        V: PartialEq + Clone,
    {
        self.values.has(&vec![value.clone()])
    }
    
    pub fn in_values(&self, values: &Vec<V>) -> bool
    where
        V: PartialEq,
    {
        values.iter().all(|value| self.values.to_vec().contains(value))
    }
    
    pub fn not_in_value(&self, value: &V) -> bool
    where
        V: PartialEq + Clone,
    {
        self.values.not_has(&vec![value.clone()])
    }
    
    pub fn not_in_values(&self, values: &Vec<V>) -> bool
    where
        V: PartialEq,
    {
        self.values.not_has(values)
    }
    
    pub fn every(&self, func: impl Fn(&K, &V) -> bool) -> &Self {
        for idx in self.keys.get_indexes() {
            let key = self.keys.get_value_by_index(idx).unwrap();
            let value = self.values.get_value_by_index(idx).unwrap();
            
            if !func(key, value) {
                return self;
            }
        }
        
        self
    }
    
    pub fn each(&mut self, func: impl Fn(&K, &V) -> V) -> &Self
    where
        K: Clone,
        V: Clone,
    {
        for idx in self.keys.get_indexes() {
            let key = self.keys.get_value_by_index(idx).unwrap();
            let value = self.values.get_value_by_index(idx).unwrap();
            
            self.values.set_value(idx, func(key, value));
        }
        
        self
    }
    
    pub fn clean(&self) -> AnyMap<K, V> {
        AnyMap::new()
    }
    
    pub fn to_string(&self, sep: Option<&str>) -> String
    where
        K: std::fmt::Display,
        V: std::fmt::Display,
    {
        let mut items:Vec<String> = vec![];
        
        for idx in self.keys.get_indexes() {
            if let Some(key) = self.keys.get_value_by_index(idx) {
                if let Some(value) = self.values.get_value_by_index(idx) {
                    items.push(format!("{key}: {value}"));
                }
            }
        }
        
        items.join(sep.unwrap_or(","))
    }
}
