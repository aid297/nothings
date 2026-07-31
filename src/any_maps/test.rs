use crate::any_maps::app::AnyMap;
use std::collections::HashMap;

// ─────────────────────────────────────────────
// new
// ─────────────────────────────────────────────

#[test]
fn test_new_empty() {
    let map: AnyMap<String, i32> = AnyMap::new();
    assert_eq!(map.to_hashmap().len(), 0);
}

// ─────────────────────────────────────────────
// from_iter
// ─────────────────────────────────────────────

#[test]
fn test_from_iter() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    assert_eq!(map.to_hashmap().len(), 3);
    assert_eq!(*map.get_value_by_key(&"a").unwrap(), 1);
    assert_eq!(*map.get_value_by_key(&"b").unwrap(), 2);
    assert_eq!(*map.get_value_by_key(&"c").unwrap(), 3);
}

#[test]
fn test_from_iter_empty() {
    let map: AnyMap<&str, i32> = AnyMap::from_iter(vec![]);
    assert_eq!(map.to_hashmap().len(), 0);
}

// ─────────────────────────────────────────────
// from_hashmap
// ─────────────────────────────────────────────

#[test]
fn test_from_hashmap() {
    let mut hm = HashMap::new();
    hm.insert("x", 10);
    hm.insert("y", 20);
    let map = AnyMap::from_hashmap(hm);
    assert_eq!(map.to_hashmap().len(), 2);
}

// ─────────────────────────────────────────────
// set_data
// ─────────────────────────────────────────────

#[test]
fn test_set_data() {
    let mut map = AnyMap::new();
    let mut hm = HashMap::new();
    hm.insert("a", 1);
    hm.insert("b", 2);
    map.set_data(hm);
    assert_eq!(map.to_hashmap().len(), 2);
}

// ─────────────────────────────────────────────
// push_datum
// ─────────────────────────────────────────────

#[test]
fn test_push_datum() {
    let mut map: AnyMap<&str, i32> = AnyMap::new();
    map.push_datum("a", 1);
    map.push_datum("b", 2);
    assert_eq!(*map.get_value_by_key(&"a").unwrap(), 1);
    assert_eq!(*map.get_value_by_key(&"b").unwrap(), 2);
}

// ─────────────────────────────────────────────
// set_value_by_index
// ─────────────────────────────────────────────

#[test]
fn test_set_value_by_index() {
    let mut map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    map.set_value_by_index(0, 100);
    assert_eq!(*map.get_value_by_key(&"a").unwrap(), 100);
}

// ─────────────────────────────────────────────
// set_value_by_key
// ─────────────────────────────────────────────

#[test]
fn test_set_value_by_key_found() {
    let mut map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    let result = map.set_value_by_key("a", 99);
    assert!(result.is_ok());
    assert_eq!(*map.get_value_by_key(&"a").unwrap(), 99);
}

#[test]
fn test_set_value_by_key_not_found() {
    let mut map = AnyMap::from_iter(vec![("a", 1)]);
    let result = map.set_value_by_key("z", 99);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────
// get_value_by_key
// ─────────────────────────────────────────────

#[test]
fn test_get_value_by_key_found() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert_eq!(*map.get_value_by_key(&"a").unwrap(), 1);
}

#[test]
fn test_get_value_by_key_not_found() {
    let map = AnyMap::from_iter(vec![("a", 1)]);
    assert!(map.get_value_by_key(&"z").is_err());
}

// ─────────────────────────────────────────────
// to_hashmap
// ─────────────────────────────────────────────

#[test]
fn test_to_hashmap() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    let hm = map.to_hashmap();
    assert_eq!(hm.len(), 2);
    assert_eq!(*hm.get("a").unwrap(), 1);
    assert_eq!(*hm.get("b").unwrap(), 2);
}

#[test]
fn test_to_hashmap_empty() {
    let map: AnyMap<&str, i32> = AnyMap::new();
    assert_eq!(map.to_hashmap().len(), 0);
}

// ─────────────────────────────────────────────
// copy
// ─────────────────────────────────────────────

#[test]
fn test_copy() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    let copied = map.copy();
    assert_eq!(*copied.get_value_by_key(&"a").unwrap(), 1);
    assert_eq!(*copied.get_value_by_key(&"b").unwrap(), 2);
}

// ─────────────────────────────────────────────
// has
// ─────────────────────────────────────────────

#[test]
fn test_has_true() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(map.has(&"a"));
}

#[test]
fn test_has_false() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(!map.has(&"z"));
}

// ─────────────────────────────────────────────
// remove_by_key (uses remove_by_index, works)
// ─────────────────────────────────────────────

#[test]
fn test_remove_by_key_found() {
    let mut map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    map.remove_by_key(&"b");
    assert!(!map.has(&"b"));
    assert_eq!(*map.get_value_by_key(&"a").unwrap(), 1);
    assert_eq!(*map.get_value_by_key(&"c").unwrap(), 3);
}

#[test]
fn test_remove_by_key_not_found() {
    let mut map = AnyMap::from_iter(vec![("a", 1)]);
    map.remove_by_key(&"z");
    assert_eq!(map.to_hashmap().len(), 1);
}

// ─────────────────────────────────────────────
// remove_by_index (uses remove_by_index, works)
// ─────────────────────────────────────────────

#[test]
fn test_remove_by_index() {
    let mut map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    map.remove_by_index(1);
    assert!(!map.has(&"b"));
    assert_eq!(map.to_hashmap().len(), 2);
}

// ─────────────────────────────────────────────
// get_index_by_key
// ─────────────────────────────────────────────

#[test]
fn test_get_index_by_key_found() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    assert_eq!(map.get_index_by_key(&"b"), Some(1));
}

#[test]
fn test_get_index_by_key_not_found() {
    let map = AnyMap::from_iter(vec![("a", 1)]);
    assert_eq!(map.get_index_by_key(&"z"), None);
}

// ─────────────────────────────────────────────
// get_indexes_by_keys
// ─────────────────────────────────────────────

#[test]
fn test_get_indexes_by_keys() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    let indexes = map.get_indexes_by_keys(&vec!["a", "c"]);
    assert_eq!(indexes, vec![0, 2]);
}

#[test]
fn test_get_indexes_by_keys_partial() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    let indexes = map.get_indexes_by_keys(&vec!["a", "z"]);
    assert_eq!(indexes, vec![0]);
}

// ─────────────────────────────────────────────
// get_indexes_by_values
// ─────────────────────────────────────────────

#[test]
fn test_get_indexes_by_values() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    let indexes = map.get_indexes_by_values(&vec![1, 3]);
    assert_eq!(indexes, vec![0, 2]);
}

// ─────────────────────────────────────────────
// in_key / not_in_key
// ─────────────────────────────────────────────

#[test]
fn test_in_key_true() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(map.in_key(&"a"));
}

#[test]
fn test_in_key_false() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(!map.in_key(&"z"));
}

#[test]
fn test_not_in_key_true() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(map.not_in_key(&"z"));
}

#[test]
fn test_not_in_key_false() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(!map.not_in_key(&"a"));
}

// ─────────────────────────────────────────────
// in_keys / not_in_keys
// ─────────────────────────────────────────────

#[test]
fn test_in_keys_true() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    assert!(map.in_keys(&vec![&"a", &"b"]));
}

#[test]
fn test_in_keys_false() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(!map.in_keys(&vec![&"a", &"z"]));
}

#[test]
fn test_not_in_keys_true() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(map.not_in_keys(&vec![&"x", &"z"]));
}

#[test]
fn test_not_in_keys_false() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(!map.not_in_keys(&vec![&"a", &"z"]));
}

// ─────────────────────────────────────────────
// in_value / not_in_value
// ─────────────────────────────────────────────

#[test]
fn test_in_value_true() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(map.in_value(&1));
}

#[test]
fn test_in_value_false() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(!map.in_value(&99));
}

#[test]
fn test_not_in_value_true() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(map.not_in_value(&99));
}

#[test]
fn test_not_in_value_false() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(!map.not_in_value(&1));
}

// ─────────────────────────────────────────────
// in_values / not_in_values
// ─────────────────────────────────────────────

#[test]
fn test_in_values_true() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    assert!(map.in_values(&vec![1, 2]));
}

#[test]
fn test_in_values_false() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(!map.in_values(&vec![1, 99]));
}

#[test]
fn test_not_in_values_true() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(map.not_in_values(&vec![88, 99]));
}

#[test]
fn test_not_in_values_false() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    assert!(!map.not_in_values(&vec![1, 99]));
}

// ─────────────────────────────────────────────
// every
// ─────────────────────────────────────────────

#[test]
fn test_every_all_match() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    // every returns &Self when all return true (no short-circuit)
    let result = map.every(|_k, _v| true);
    let _ = result;
}

#[test]
fn test_every_short_circuit() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);
    // Should short-circuit on first false, but returns &Self either way
    let result = map.every(|k, _v| *k != "a");
    // Just verify it returns without panic
    let _ = result;
}

#[test]
fn test_every_empty() {
    let map: AnyMap<&str, i32> = AnyMap::new();
    let result = map.every(|_, _| false);
    let _ = result;
}

// ─────────────────────────────────────────────
// each
// ─────────────────────────────────────────────

#[test]
fn test_each() {
    let mut map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    map.each(|_k, v| v * 10);
    assert_eq!(*map.get_value_by_key(&"a").unwrap(), 10);
    assert_eq!(*map.get_value_by_key(&"b").unwrap(), 20);
}

#[test]
fn test_each_empty() {
    let mut map: AnyMap<&str, i32> = AnyMap::new();
    map.each(|_k, v| v * 10);
    assert_eq!(map.to_hashmap().len(), 0);
}

// ─────────────────────────────────────────────
// clean
// ─────────────────────────────────────────────

#[test]
fn test_clean() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    let empty: AnyMap<&str, i32> = map.clean();
    assert_eq!(empty.to_hashmap().len(), 0);
}

// ─────────────────────────────────────────────
// to_string
// ─────────────────────────────────────────────

#[test]
fn test_to_string_default_sep() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    let s = map.to_string(None);
    assert_eq!(s, "a: 1,b: 2");
}

#[test]
fn test_to_string_custom_sep() {
    let map = AnyMap::from_iter(vec![("a", 1), ("b", 2)]);
    let s = map.to_string(Some(" | "));
    assert_eq!(s, "a: 1 | b: 2");
}

#[test]
fn test_to_string_empty() {
    let map: AnyMap<&str, i32> = AnyMap::new();
    let s = map.to_string(None);
    assert_eq!(s, "");
}
