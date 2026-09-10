# AnyMap — 有序键值对映射

`AnyMap<K, V>` 是基于两个平行 `AnySlice`（keys / values）实现的有序键值对映射结构，保留插入顺序。

```rust
use nothings::any_maps::app::AnyMap;

let mut map = AnyMap::with_iter(vec![("alice", 90), ("bob", 85), ("charlie", 92)]);

assert_eq!(map.get_value_by_key(&"alice").unwrap(), 90);
assert!(map.has(&"bob"));
assert!(map.in_value(&85));
assert_eq!(map.len(), 3);

map.push_datum("dave", 88);
map.each(|_key, value| value + 1);

println!("{}", map.to_string(Some(", ")));
// alice: 91, bob: 86, charlie: 93, dave: 89
```

## API 一览

共 34 个公开方法。下表的「约束」列列出该方法要求的 trait bound，未列出则无额外约束。

### 构造与数据访问

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `new()` | `-> Self` | 创建空的 `AnyMap` | |
| `with_iter(iter)` | `I: IntoIterator<Item = (K, V)>` `-> Self` | 从迭代器创建，**保留迭代顺序** | |
| `with_hashmap(hash_map)` | `HashMap<K, V> -> Self` | 从 `HashMap` 创建，顺序为 `HashMap` 的迭代顺序（不确定） | |
| `set_data(&mut self, hash_map)` | `HashMap<K, V> -> &mut Self` | 用 `HashMap` 替换全部数据 | |
| `push_datum(&mut self, key, value)` | `K, V -> &mut Self` | 追加一对键值，支持链式调用。**不做去重**，重复 key 会出现多次 | |
| `to_hashmap(&self)` | `-> HashMap<K, V>` | 转换为 `HashMap` | `K: Clone + Eq + Hash`, `V: Clone` |
| `to_entries(&self)` | `-> Vec<(K, V)>` | 按插入顺序导出全部键值对 | `K: Clone`, `V: Clone` |
| `copy(&self)` | `-> Self` | 深拷贝 | `K: Clone`, `V: Clone` |
| `clean(&mut self)` | `-> &mut Self` | **原地清空**自身并返回 `&mut Self`（不返回新的空 `AnyMap`） | |
| `len(&self)` | `-> usize` | 键值对数量 | |

### 值操作

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `set_value_by_index(&mut self, index, value)` | `usize, V -> &mut Self` | 按索引设置值。**索引越界会 panic** | `V: Clone` |
| `set_value_by_key(&mut self, key, value)` | `K, V -> Result<&mut Self, Error>` | 按 key 设置值，key 不存在返回 `Err(NotFound)` | `K: PartialEq`, `V: Clone` |
| `get_value_by_key(&self, key)` | `&K -> Result<V, Error>` | 按 key 获取值。**返回克隆出的 `V`（非 `&V`）**，不需要解引用 | `K: PartialEq`, `V: Clone` |
| `get_index_by_key(&self, key)` | `&K -> Option<usize>` | 按 key 获取索引 | `K: PartialEq` |
| `get_indexes_by_keys(&self, keys)` | `&[K] -> Vec<usize>` | 批量获取索引；跳过不存在的 key，结果已排序去重 | `K: PartialEq` |
| `get_indexes_by_values(&self, values)` | `&[V] -> Vec<usize>` | 按 value 批量获取索引；跳过不存在的 value，结果已排序去重 | `V: PartialEq` |

### 查询与判断

| 方法 | 语义 | 说明 |
|------|------|------|
| `has(key)` | key 存在 | `K: PartialEq + Clone` |
| `in_key(key)` | key 存在 | 与 `has` 等价 |
| `in_keys(keys)` | **所有** key 都存在 | 底层用 `all()`，只要有一个缺失即返回 `false` |
| `not_in_key(key)` | key 不存在 | |
| `not_in_keys(keys)` | **所有** key 都不存在 | 只要有一个存在即返回 `false` |
| `in_value(value)` | value 存在 | `V: PartialEq + Clone` |
| `in_values(values)` | **所有** value 都存在 | 底层用 `all()` |
| `not_in_value(value)` | value 不存在 | |
| `not_in_values(values)` | **所有** value 都不存在 | |

### 删除与过滤

| 方法 | 状态 | 说明 |
|------|------|------|
| `remove_by_key(&mut self, key: &K)` | 可用 | 按 key 移除；key 不存在时静默返回 |
| `remove_by_index(&mut self, index: usize)` | 可用 | 按索引移除；越界安全，静默返回 |
| `remove_by_keys(&mut self, keys: &Vec<K>)` | ⚠️ 当前无效 | 见下方「已知问题」 |
| `remove_by_values(&mut self, values: &Vec<V>)` | ⚠️ 当前无效 | 见下方「已知问题」 |
| `remove_empty(&mut self)` | ⚠️ 当前无效 | 见下方「已知问题」。返回类型为 `&Self` |
| `filter(&mut self, f: impl Fn(&K, &V) -> bool)` | ⚠️ 当前无效 | 见下方「已知问题」 |

### 遍历与转换

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `each(&mut self, func)` | `impl Fn(&K, &V) -> V` `-> &mut Self` | 对每个键值对计算新值并原地更新 values | `K: Clone`, `V: Clone` |
| `every(&self, func)` | `impl Fn(&K, &V) -> bool` `-> &Self` | ⚠️ **不返回 `bool`**，见下方「已知问题」 | `K: Clone`, `V: Clone` |
| `to_string(&self, sep)` | `Option<&str> -> String` | 格式为 `key: value`，用 `sep` 连接（默认 `,`） | `K: Display + Clone`, `V: Display + Clone` |

## 已知问题

以下方法当前**不会修改数据**，根因是 `AnySlice::remove_by_indexes` 中使用了惰性迭代器但未消费：

```rust
// src/any_slices/app.rs:347
pub fn remove_by_indexes(&mut self, indexes: &Vec<usize>) -> &mut Self {
    let _ = indexes.iter().map(|index| self.data.remove(*index)); // map 是惰性的，立即被丢弃
    self
}
```

受影响的方法：`filter`、`remove_by_keys`、`remove_by_values`、`remove_empty`（全部依赖 `remove_by_indexes`）。调用后长度和内容均不变。

`every` 的实现在匹配与不匹配两条分支上都返回 `&Self`，无法用作谓词判断。

临时替代写法：

```rust
// 代替 filter / every
let kept: Vec<(&str, i32)> = map.to_entries()
    .into_iter()
    .filter(|(_k, v)| *v >= 90)
    .collect();
let map = AnyMap::with_iter(kept);

let all_match = map.to_entries().iter().all(|(_k, v)| *v >= 90);
```

## 示例

### 从 HashMap 创建并操作

```rust
use nothings::any_maps::app::AnyMap;
use std::collections::HashMap;

let mut hm = HashMap::new();
hm.insert("math", 95);
hm.insert("english", 88);

let mut scores = AnyMap::with_hashmap(hm);

scores.push_datum("history", 90);
scores.set_value_by_key("english", 91).unwrap();

assert_eq!(scores.get_value_by_key(&"math").unwrap(), 95);
assert_eq!(scores.get_value_by_key(&"english").unwrap(), 91);
assert_eq!(scores.len(), 3);
```

### 查询与遍历

```rust
use nothings::any_maps::app::AnyMap;

let mut map = AnyMap::with_iter(vec![("attr", 1), ("b", 2), ("c", 3)]);

assert!(map.in_key(&"attr"));
assert!(!map.in_keys(&vec!["attr", "zzz"])); // 需要全部存在
assert!(map.in_values(&vec![1, 3]));         // 需要全部存在
assert!(!map.not_in_key(&"b"));

// 遍历转换：每个 value 乘以 10
map.each(|_k, v| v * 10);
assert_eq!(map.get_value_by_key(&"attr").unwrap(), 10);

// 按插入顺序导出
assert_eq!(map.to_entries(), vec![("attr", 10), ("b", 20), ("c", 30)]);
```

### 删除

```rust
use nothings::any_maps::app::AnyMap;

let mut map = AnyMap::with_iter(vec![("attr", 1), ("b", 2), ("c", 3)]);

map.remove_by_key(&"b");
assert_eq!(map.len(), 2);

map.remove_by_index(0); // 移除 "attr"
assert_eq!(map.to_entries(), vec![("c", 3)]);

map.remove_by_index(99); // 越界安全，不 panic
assert_eq!(map.len(), 1);
```

## 备注

- `app.rs` 中声明了 `AnyMapTrait`，但 `AnyMap` 并未实现它，仅作为接口说明存在。请勿在代码中 `use` 该 trait。
- `Default` 仅为 `AnyMap<(), ()>` 实现，其他类型参数请用 `AnyMap::new()`。
- `AnyMap` 不是 `std::collections::HashMap` 的替代品：查找为 O(n) 线性扫描，适合数据量小且需要保留插入顺序的场景。
