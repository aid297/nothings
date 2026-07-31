# AnySlice — 增强版 Vec

`AnySlice<T>` 是对 `Vec<T>` 的增强封装，提供 43 个方法，支持丰富的链式操作。

## 快速开始

```rust
use aid::anySlices::app::AnySlice;

let mut slice = AnySlice::new(vec![1, 2, 3, 4, 5]);

// 查询
assert!(slice.has(&3));
assert_eq!(slice.first(), Some(&1));
assert_eq!(slice.last(), Some(&5));

// 链式操作
slice.push(6).push(7);

// 集合运算
let a = AnySlice::new(vec![1, 2, 3]);
let b = vec![2, 3, 4];
let union = a.union(&b);         // [1, 2, 3, 4]
let intersection = a.intersection(&b); // [2, 3]
let difference = a.difference(&b);     // [1]
```

## API 一览

### 构造与数据访问

| 方法 | 说明 |
|------|------|
| `new(vec)` | 从 `Vec<T>` 创建 `AnySlice` |
| `get_data(self)` | 消耗自身，返回内部 `Vec<T>` |
| `to_vec(&self)` | 返回内部 `Vec<T>` 的引用 |
| `set_data(&mut self, data)` | 替换内部数据 |

### 元素操作

| 方法 | 说明 |
|------|------|
| `set_value(index, value)` | 设置指定索引的值，支持链式调用 |
| `push(value)` | 追加单个元素，支持链式调用 |
| `append(values)` | 追加多个元素，支持链式调用 |
| `filter(predicate)` | 原地过滤，仅保留满足条件的元素 |
| `each(func)` | 对每个元素执行转换函数并原地更新 |
| `sort(func)` | 按自定义比较函数排序 |
| `clean()` | 清空所有数据 |
| `remove_by_index(index)` | 按索引移除元素（返回新 `AnySlice`） |
| `remove_empty()` | 移除所有默认值元素（如 `0`、`""`） |

### 查询与判断

| 方法 | 说明 |
|------|------|
| `empty()` | 是否为空 |
| `not_empty()` | 是否非空 |
| `has(value)` | 是否包含指定值 |
| `not_has(value)` | 是否不包含指定值 |
| `len()` | 元素个数 |
| `len_without_empty()` | 非默认值元素个数 |
| `all_empty()` | 是否所有元素都是默认值 |
| `any_empty()` | 是否存在默认值元素 |
| `every(func)` | 是否所有元素都满足条件 |

### 元素获取

| 方法 | 说明 |
|------|------|
| `get_value(index)` | 获取指定索引的引用（`Option<&T>`） |
| `get_value_ptr(index)` | 获取指定索引的原始指针 |
| `get_value_default(index, default)` | 获取指定索引的值，越界时返回默认值 |
| `get_values(indexes)` | 按多个索引批量获取元素 |
| `get_values_by_slicer(slicer)` | 按 `AnySlice<usize>` 索引批量获取 |
| `first()` | 获取第一个元素 |
| `last()` | 获取最后一个元素 |

### 索引查找

| 方法 | 说明 |
|------|------|
| `get_indexes()` | 获取所有索引 `[0, 1, ..., n-1]` |
| `get_index_by_value(value)` | 按值查找第一个匹配的索引 |
| `get_indexes_by_values(values)` | 按多个值批量查找索引 |

### 集合运算

| 方法 | 说明 |
|------|------|
| `union(other)` | 并集（参数为 `&Vec<T>`） |
| `union_slicer(other)` | 并集（参数为 `&AnySlice<T>`） |
| `intersection(other)` | 交集（参数为 `&Vec<T>`） |
| `intersection_slicer(other)` | 交集（参数为 `&AnySlice<T>`） |
| `difference(other)` | 差集（参数为 `&Vec<T>`） |
| `difference_slicer(other)` | 差集（参数为 `&AnySlice<T>`） |

### 转换与工具

| 方法 | 说明 |
|------|------|
| `copy()` | 深拷贝 |
| `chunk(size)` | 按指定大小分块 |
| `pluck(func)` | 对每个元素执行映射函数，收集为 `Vec` |
| `shuffle()` | 随机打乱（返回新 `AnySlice`） |
| `to_string(sep)` | 转为字符串，可指定分隔符（默认 `,`） |

## 链式调用示例

### 排序 + 过滤 + 转字符串

```rust
let mut slice = AnySlice::new(vec![5, 3, 1, 4, 2]);

slice.sort(|a, b| a.cmp(b))
     .filter(|x| *x > 2);

println!("{}", slice.to_string(Some(" -> ")));
// 输出: 3 -> 4 -> 5
```

### 集合运算

```rust
let a = AnySlice::new(vec![1, 2, 3, 4]);
let b = AnySlice::new(vec![3, 4, 5, 6]);

let result = a.union_slicer(&b)
              .difference_slicer(&AnySlice::new(vec![5]));

println!("{:?}", result.to_vec());
// 输出: [1, 2, 3, 4, 6]
```

### pluck 提取字段

```rust
struct User { name: String, age: u8 }

let users = AnySlice::new(vec![
    User { name: "Alice".into(), age: 30 },
    User { name: "Bob".into(), age: 25 },
]);

let names: Vec<String> = users.pluck(|u| u.name.clone());
// ["Alice", "Bob"]
```
