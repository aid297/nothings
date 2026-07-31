# AnyMap — 有序键值对映射

`AnyMap<K, V>` 是基于两个平行 `AnySlice`（keys/values）实现的有序键值对映射结构，保留插入顺序。

## 快速开始

```rust
use aid::anyMaps::app::AnyMap;

// 从迭代器创建
let mut map = AnyMap::from_iter(vec![("alice", 90), ("bob", 85), ("charlie", 92)]);

// 查询
assert_eq!(*map.get_value_by_key(&"alice").unwrap(), 90);
assert!(map.has(&"bob"));
assert!(map.in_value(&85));

// 链式操作
map.push_datum("dave", 88);

// 按条件过滤
map.filter(|key, value| *value >= 90);

// 遍历转换
println!("{}", map.to_string(Some(", ")));
// 输出: alice: 90, charlie: 92
```

## API 一览

### 构造与数据访问

| 方法 | 说明 |
|------|------|
| `new()` | 创建空的 `AnyMap` |
| `from_iter(iter)` | 从迭代器 `IntoIterator<Item=(K, V)>` 创建 |
| `from_hashmap(hash_map)` | 从 `HashMap<K, V>` 创建 |
| `set_data(hash_map)` | 用 `HashMap` 替换全部数据 |
| `push_datum(key, value)` | 追加一对键值，支持链式调用 |
| `to_hashmap()` | 转换为 `HashMap<K, V>` |
| `copy()` | 深拷贝 |
| `clean()` | 返回空的 `AnyMap` |

### 值操作

| 方法 | 说明 |
|------|------|
| `set_value_by_index(index, value)` | 按索引设置值 |
| `set_value_by_key(key, value)` | 按 key 设置值（key 不存在返回 `Err`） |
| `get_value_by_key(key)` | 按 key 获取值（返回 `Result<&V, Error>`） |
| `get_index_by_key(key)` | 按 key 获取索引（返回 `Option<usize>`） |
| `get_indexes_by_keys(keys)` | 按多个 key 批量获取索引 |
| `get_indexes_by_values(values)` | 按多个 value 批量获取索引 |

### 查询与判断

| 方法 | 说明 |
|------|------|
| `has(key)` | 是否包含指定 key |
| `in_key(key)` | key 是否存在 |
| `in_keys(keys)` | 是否包含任意一个 key |
| `not_in_key(key)` | key 是否不存在 |
| `not_in_keys(keys)` | 是否不包含任何 key |
| `in_value(value)` | value 是否存在 |
| `in_values(values)` | 是否包含任意一个 value |
| `not_in_value(value)` | value 是否不存在 |
| `not_in_values(values)` | 是否不包含任何 value |
| `every(func)` | 是否所有键值对都满足条件 |

### 删除操作

| 方法 | 说明 |
|------|------|
| `remove_by_key(key)` | 按 key 移除键值对 |
| `remove_by_index(index)` | 按索引移除键值对（越界安全） |
| `remove_by_keys(keys)` | 按多个 key 批量移除 |
| `remove_by_values(values)` | 按多个 value 批量移除 |
| `remove_empty()` | 移除值为默认值的键值对 |
| `filter(func)` | 按条件过滤，保留满足条件的键值对 |

### 遍历与转换

| 方法 | 说明 |
|------|------|
| `each(func)` | 对每个键值对执行转换函数，原地更新值 |
| `to_string(sep)` | 转为字符串，可指定分隔符（默认 `,`） |

## 链式调用示例

### 从 HashMap 创建并操作

```rust
use aid::anyMaps::app::AnyMap;
use std::collections::HashMap;

let mut hm = HashMap::new();
hm.insert("math", 95);
hm.insert("english", 88);
hm.insert("science", 92);

let mut scores = AnyMap::from_hashmap(hm);

// 追加
scores.push_datum("history", 90);

// 按 key 修改值
scores.set_value_by_key("english", 91);

// 按条件过滤（保留 >= 90 的科目）
scores.filter(|_key, value| *value >= 90);

// 转字符串输出
println!("{}", scores.to_string(Some(" | ")));
// 输出: math: 95 | science: 92 | history: 90
```

### 查询与判断

```rust
let map = AnyMap::from_iter(vec![("a", 1), ("b", 2), ("c", 3)]);

assert!(map.in_key(&"a"));           // key "a" 存在
assert!(map.in_values(&vec![1, 3])); // value 1 或 3 存在
assert!(!map.not_in_key(&"b"));      // key "b" 存在

// 遍历转换：每个 value 乘以 10
map.each(|_k, v| v * 10);
assert_eq!(*map.get_value_by_key(&"a").unwrap(), 10);
```
