use crate::any_slices::app::AnySlice;

// ─────────────────────────────────────────────
// new / get_data / to_vec
// ─────────────────────────────────────────────

#[test]
fn test_new_and_get_data() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert_eq!(slice.get_data(), vec![1, 2, 3]);
}

#[test]
fn test_new_empty() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert!(slice.get_data().is_empty());
}

#[test]
fn test_to_vec() {
    let slice = AnySlice::new(vec![10, 20]);
    assert_eq!(slice.to_vec(), &vec![10, 20]);
}

// ─────────────────────────────────────────────
// set_data
// ─────────────────────────────────────────────

#[test]
fn test_set_data() {
    let mut slice = AnySlice::new(vec![1, 2]);
    slice.set_data(vec![9, 8, 7]);
    assert_eq!(slice.get_data(), vec![9, 8, 7]);
}

// ─────────────────────────────────────────────
// set_value
// ─────────────────────────────────────────────

#[test]
fn test_set_value() {
    let mut slice = AnySlice::new(vec![1, 2, 3]);
    slice.set_value(1, 99);
    assert_eq!(slice.get_data(), vec![1, 99, 3]);
}

#[test]
fn test_set_value_chain() {
    let mut slice = AnySlice::new(vec![0, 0, 0]);
    slice.set_value(0, 1).set_value(2, 3);
    assert_eq!(slice.get_data(), vec![1, 0, 3]);
}

// ─────────────────────────────────────────────
// empty / not_empty
// ─────────────────────────────────────────────

#[test]
fn test_empty_true() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert!(slice.empty());
}

#[test]
fn test_empty_false() {
    let slice = AnySlice::new(vec![1]);
    assert!(!slice.empty());
}

#[test]
fn test_not_empty_true() {
    let slice = AnySlice::new(vec![1]);
    assert!(slice.not_empty());
}

#[test]
fn test_not_empty_false() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert!(!slice.not_empty());
}

// ─────────────────────────────────────────────
// has / not_has
// ─────────────────────────────────────────────

#[test]
fn test_has_true() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert!(slice.has(&vec![2]));
}

#[test]
fn test_has_false() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert!(!slice.has(&vec![99]));
}

#[test]
fn test_not_has_true() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert!(slice.not_has(&vec![99]));
}

#[test]
fn test_not_has_false() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert!(!slice.not_has(&vec![2]));
}

// ─────────────────────────────────────────────
// get_value
// ─────────────────────────────────────────────

#[test]
fn test_get_value_some() {
    let slice = AnySlice::new(vec![10, 20, 30]);
    assert_eq!(slice.get_value_by_index(1), Some(&20));
}

#[test]
fn test_get_value_none() {
    let slice = AnySlice::new(vec![10]);
    assert_eq!(slice.get_value_by_index(5), None);
}

// ─────────────────────────────────────────────
// get_value_ptr
// ─────────────────────────────────────────────

#[test]
fn test_get_value_ptr_some() {
    let slice = AnySlice::new(vec![10, 20, 30]);
    let ptr = slice.get_value_ptr(1);
    assert!(ptr.is_some());
    unsafe {
        assert_eq!(*ptr.unwrap(), 20);
    }
}

#[test]
fn test_get_value_ptr_none() {
    let slice = AnySlice::new(vec![10]);
    assert!(slice.get_value_ptr(10).is_none());
}

// ─────────────────────────────────────────────
// get_value_default
// ─────────────────────────────────────────────

#[test]
fn test_get_value_default_exists() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert_eq!(slice.get_value_default(1, 999), 2);
}

#[test]
fn test_get_value_default_fallback() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert_eq!(slice.get_value_default(99, 999), 999);
}

// ─────────────────────────────────────────────
// get_values
// ─────────────────────────────────────────────

#[test]
fn test_get_values() {
    let slice = AnySlice::new(vec![10, 20, 30, 40]);
    let result = slice.get_values(&[0, 2, 3]);
    assert_eq!(result, vec![&10, &30, &40]);
}

#[test]
fn test_get_values_out_of_range_ignored() {
    let slice = AnySlice::new(vec![10, 20]);
    let result = slice.get_values(&[0, 5, 1]);
    assert_eq!(result, vec![&10, &20]);
}

// ─────────────────────────────────────────────
// get_values_by_slicer
// ─────────────────────────────────────────────

#[test]
fn test_get_values_by_slicer() {
    let slice = AnySlice::new(vec![10, 20, 30, 40]);
    let slicer = AnySlice::new(vec![1, 3]);
    let result = slice.get_values_by_slicer(&slicer);
    assert_eq!(result, vec![&20, &40]);
}

// ─────────────────────────────────────────────
// first / last
// ─────────────────────────────────────────────

#[test]
fn test_first_some() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert_eq!(slice.first(), Some(&1));
}

#[test]
fn test_first_none() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert_eq!(slice.first(), None);
}

#[test]
fn test_last_some() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert_eq!(slice.last(), Some(&3));
}

#[test]
fn test_last_none() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert_eq!(slice.last(), None);
}

// ─────────────────────────────────────────────
// get_indexes
// ─────────────────────────────────────────────

#[test]
fn test_get_indexes() {
    let slice = AnySlice::new(vec![10, 20, 30]);
    assert_eq!(slice.get_indexes(), vec![0, 1, 2]);
}

#[test]
fn test_get_indexes_empty() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert_eq!(slice.get_indexes(), Vec::<usize>::new());
}

// ─────────────────────────────────────────────
// get_index_by_value
// ─────────────────────────────────────────────

#[test]
fn test_get_index_by_value_found() {
    let slice = AnySlice::new(vec![10, 20, 30]);
    assert_eq!(slice.get_index_by_value(&20), Some(1));
}

#[test]
fn test_get_index_by_value_not_found() {
    let slice = AnySlice::new(vec![10, 20, 30]);
    assert_eq!(slice.get_index_by_value(&99), None);
}

// ─────────────────────────────────────────────
// get_indexes_by_values
// ─────────────────────────────────────────────

#[test]
fn test_get_indexes_by_values() {
    let slice = AnySlice::new(vec![10, 20, 30, 40]);
    let result = slice.get_indexes_by_values(&vec![20, 40]);
    assert_eq!(result, vec![1, 3]);
}

// ─────────────────────────────────────────────
// shuffle
// ─────────────────────────────────────────────

#[test]
fn test_shuffle_same_length() {
    let slice = AnySlice::new(vec![1, 2, 3, 4, 5]);
    let shuffled = slice.shuffle();
    assert_eq!(shuffled.len(), slice.len());
}

#[test]
fn test_shuffle_contains_same_elements() {
    let slice = AnySlice::new(vec![1, 2, 3, 4, 5]);
    let shuffled = slice.shuffle();
    let mut orig = shuffled.get_data();
    orig.sort();
    assert_eq!(orig, vec![1, 2, 3, 4, 5]);
}

// ─────────────────────────────────────────────
// len
// ─────────────────────────────────────────────

#[test]
fn test_len() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert_eq!(slice.len(), 3);
}

#[test]
fn test_len_empty() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert_eq!(slice.len(), 0);
}

// ─────────────────────────────────────────────
// len_without_empty
// ─────────────────────────────────────────────

#[test]
fn test_len_without_empty() {
    let slice = AnySlice::new(vec![0, 1, 0, 2, 0]);
    assert_eq!(slice.len_without_empty(), 2);
}

#[test]
fn test_len_without_empty_all_non_default() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert_eq!(slice.len_without_empty(), 3);
}

#[test]
fn test_len_without_empty_string() {
    let slice = AnySlice::new(vec!["".to_string(), "a".to_string(), "".to_string()]);
    assert_eq!(slice.len_without_empty(), 1);
}

// ─────────────────────────────────────────────
// remove_empty
// ─────────────────────────────────────────────

#[test]
fn test_remove_empty() {
    let mut slice = AnySlice::new(vec![0, 1, 0, 2, 0]);
    slice.remove_empty();
    assert_eq!(slice.get_data(), vec![1, 2]);
}

#[test]
fn test_remove_empty_string() {
    let mut slice = AnySlice::new(vec!["".to_string(), "hello".to_string(), "".to_string()]);
    slice.remove_empty();
    assert_eq!(slice.get_data(), vec!["hello".to_string()]);
}

// ─────────────────────────────────────────────
// append
// ─────────────────────────────────────────────

#[test]
fn test_append() {
    let mut slice = AnySlice::new(vec![1, 2]);
    slice.append(vec![3, 4]);
    assert_eq!(slice.get_data(), vec![1, 2, 3, 4]);
}

#[test]
fn test_append_empty_vec() {
    let mut slice = AnySlice::new(vec![1, 2]);
    slice.append(vec![]);
    assert_eq!(slice.get_data(), vec![1, 2]);
}

// ─────────────────────────────────────────────
// push
// ─────────────────────────────────────────────

#[test]
fn test_push() {
    let mut slice = AnySlice::new(vec![1, 2]);
    slice.push(3);
    assert_eq!(slice.get_data(), vec![1, 2, 3]);
}

#[test]
fn test_push_chain() {
    let mut slice = AnySlice::new(vec![]);
    slice.push(1).push(2).push(3);
    assert_eq!(slice.get_data(), vec![1, 2, 3]);
}

// ─────────────────────────────────────────────
// filter
// ─────────────────────────────────────────────

#[test]
fn test_filter() {
    let mut slice = AnySlice::new(vec![1, 2, 3, 4, 5]);
    slice.filter(|x| x % 2 == 0);
    assert_eq!(slice.get_data(), vec![2, 4]);
}

#[test]
fn test_filter_all_removed() {
    let mut slice = AnySlice::new(vec![1, 3, 5]);
    slice.filter(|x| x % 2 == 0);
    assert!(slice.get_data().is_empty());
}

// ─────────────────────────────────────────────
// all_empty
// ─────────────────────────────────────────────

#[test]
fn test_all_empty_true() {
    let slice = AnySlice::new(vec![0, 0, 0]);
    assert!(slice.all_empty());
}

#[test]
fn test_all_empty_false() {
    let slice = AnySlice::new(vec![0, 1, 0]);
    assert!(!slice.all_empty());
}

#[test]
fn test_all_empty_empty_vec() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert!(slice.all_empty());
}

// ─────────────────────────────────────────────
// any_empty
// ─────────────────────────────────────────────

#[test]
fn test_any_empty_true() {
    let slice = AnySlice::new(vec![1, 0, 2]);
    assert!(slice.any_empty());
}

#[test]
fn test_any_empty_false() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert!(!slice.any_empty());
}

// ─────────────────────────────────────────────
// copy
// ─────────────────────────────────────────────

#[test]
fn test_copy() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    let copied = slice.copy();
    assert_eq!(copied.to_vec(), &vec![1, 2, 3]);
    // 修改副本不影响原始
    drop(slice);
    assert_eq!(copied.to_vec(), &vec![1, 2, 3]);
}

// ─────────────────────────────────────────────
// chunk
// ─────────────────────────────────────────────

#[test]
fn test_chunk_even() {
    let slice = AnySlice::new(vec![1, 2, 3, 4]);
    let chunks = slice.chunk(2);
    assert_eq!(chunks, vec![vec![1, 2], vec![3, 4]]);
}

#[test]
fn test_chunk_uneven() {
    let slice = AnySlice::new(vec![1, 2, 3, 4, 5]);
    let chunks = slice.chunk(2);
    assert_eq!(chunks, vec![vec![1, 2], vec![3, 4], vec![5]]);
}

#[test]
fn test_chunk_larger_than_data() {
    let slice = AnySlice::new(vec![1, 2]);
    let chunks = slice.chunk(10);
    assert_eq!(chunks, vec![vec![1, 2]]);
}

// ─────────────────────────────────────────────
// pluck
// ─────────────────────────────────────────────

#[test]
fn test_pluck() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    let result: Vec<i32> = slice.pluck(|x| x * 10);
    assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn test_pluck_type_change() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    let result: Vec<String> = slice.pluck(|x| x.to_string());
    assert_eq!(result, vec!["1", "2", "3"]);
}

// ─────────────────────────────────────────────
// intersection / intersection_slicer
// ─────────────────────────────────────────────

#[test]
fn test_intersection() {
    let slice = AnySlice::new(vec![1, 2, 3, 4]);
    let result = slice.intersection(&vec![2, 4, 6]);
    assert_eq!(result.get_data(), vec![2, 4]);
}

#[test]
fn test_intersection_no_overlap() {
    let slice = AnySlice::new(vec![1, 2]);
    let result = slice.intersection(&vec![3, 4]);
    assert!(result.get_data().is_empty());
}

#[test]
fn test_intersection_slicer() {
    let slice = AnySlice::new(vec![1, 2, 3, 4]);
    let other = AnySlice::new(vec![2, 4, 6]);
    let result = slice.intersection_slicer(&other);
    assert_eq!(result.get_data(), vec![2, 4]);
}

// ─────────────────────────────────────────────
// difference / difference_slicer
// ─────────────────────────────────────────────

#[test]
fn test_difference() {
    let slice = AnySlice::new(vec![1, 2, 3, 4]);
    let result = slice.difference(&vec![2, 4]);
    assert_eq!(result.get_data(), vec![1, 3]);
}

#[test]
fn test_difference_no_removal() {
    let slice = AnySlice::new(vec![1, 2]);
    let result = slice.difference(&vec![3, 4]);
    assert_eq!(result.get_data(), vec![1, 2]);
}

#[test]
fn test_difference_slicer() {
    let slice = AnySlice::new(vec![1, 2, 3, 4]);
    let other = AnySlice::new(vec![2, 4]);
    let result = slice.difference_slicer(&other);
    assert_eq!(result.get_data(), vec![1, 3]);
}

// ─────────────────────────────────────────────
// union / union_slicer
// ─────────────────────────────────────────────

#[test]
fn test_union() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    let result = slice.union(&vec![2, 3, 4]);
    assert_eq!(result.get_data(), vec![1, 2, 3, 4]);
}

#[test]
fn test_union_no_overlap() {
    let slice = AnySlice::new(vec![1, 2]);
    let result = slice.union(&vec![3, 4]);
    assert_eq!(result.get_data(), vec![1, 2, 3, 4]);
}

#[test]
fn test_union_full_overlap() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    let result = slice.union(&vec![1, 2, 3]);
    assert_eq!(result.get_data(), vec![1, 2, 3]);
}

#[test]
fn test_union_slicer() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    let other = AnySlice::new(vec![2, 3, 4]);
    let result = slice.union_slicer(&other);
    assert_eq!(result.get_data(), vec![1, 2, 3, 4]);
}

// ─────────────────────────────────────────────
// remove_by_index
// ─────────────────────────────────────────────

#[test]
fn test_remove_by_index() {
    let mut slice = AnySlice::new(vec![1, 2, 3, 4]);
    slice.remove_by_index(&1);
    assert_eq!(*slice.to_vec(), vec![1, 3, 4]);
}

#[test]
fn test_remove_by_index_out_of_range() {
    let mut slice = AnySlice::new(vec![1, 2, 3]);
    slice.remove_by_index(&10);
    assert_eq!(*slice.get_data(), vec![1, 2, 3]);
}

#[test]
fn test_remove_by_index_first() {
    let mut slice = AnySlice::new(vec![1, 2, 3]);
    slice.remove_by_index(&0);
    assert_eq!(*slice.get_data(), vec![2, 3]);
}

#[test]
fn test_remove_by_index_last() {
    let mut slice = AnySlice::new(vec![1, 2, 3]);
    slice.remove_by_index(&2);
    assert_eq!(*slice.get_data(), vec![1, 2]);
}

// ─────────────────────────────────────────────
// every
// ─────────────────────────────────────────────

#[test]
fn test_every_true() {
    let slice = AnySlice::new(vec![2, 4, 6]);
    assert!(slice.every(|_,x| x % 2 == 0));
}

#[test]
fn test_every_false() {
    let slice = AnySlice::new(vec![2, 3, 6]);
    assert!(!slice.every(|_,x| x % 2 == 0));
}

#[test]
fn test_every_empty() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert!(slice.every(|_,x| x % 2 == 0));
}

// ─────────────────────────────────────────────
// each
// ─────────────────────────────────────────────

#[test]
fn test_each() {
    let mut slice = AnySlice::new(vec![1, 2, 3]);
    slice.each(|_,x| x * 10);
    assert_eq!(slice.get_data(), vec![10, 20, 30]);
}

// ─────────────────────────────────────────────
// sort
// ─────────────────────────────────────────────

#[test]
fn test_sort_ascending() {
    let mut slice = AnySlice::new(vec![3, 1, 4, 1, 5]);
    slice.sort(|a, b| a.cmp(b));
    assert_eq!(slice.get_data(), vec![1, 1, 3, 4, 5]);
}

#[test]
fn test_sort_descending() {
    let mut slice = AnySlice::new(vec![3, 1, 4]);
    slice.sort(|a, b| b.cmp(a));
    assert_eq!(slice.get_data(), vec![4, 3, 1]);
}

// ─────────────────────────────────────────────
// clean
// ─────────────────────────────────────────────

#[test]
fn test_clean() {
    let mut slice = AnySlice::new(vec![1, 2, 3]);
    slice.clean();
    assert!(slice.get_data().is_empty());
}

// ─────────────────────────────────────────────
// to_string
// ─────────────────────────────────────────────

#[test]
fn test_to_string_default_sep() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert_eq!(slice.to_string(None), "1,2,3");
}

#[test]
fn test_to_string_custom_sep() {
    let slice = AnySlice::new(vec![1, 2, 3]);
    assert_eq!(slice.to_string(Some(" | ")), "1 | 2 | 3");
}

#[test]
fn test_to_string_empty() {
    let slice: AnySlice<i32> = AnySlice::new(vec![]);
    assert_eq!(slice.to_string(None), "");
}

#[test]
fn test_to_string_single() {
    let slice = AnySlice::new(vec![42]);
    assert_eq!(slice.to_string(Some("-")), "42");
}
