# AnySlice — 增强版 Vec

`AnySlice<T>` 是对 `Vec<T>` 的增强封装，提供 45 个方法，支持丰富的链式操作。

```rust
use nothings::any_slices::app::AnySlice;

let mut slice = AnySlice::new(vec![1, 2, 3, 4, 5]);

// 查询：has 的参数是 &Vec<T>
assert!(slice.has(&vec![3]));
// first / last 返回克隆出的 Option<T>，不是 Option<&T>
assert_eq!(slice.first(), Some(1));
assert_eq!(slice.last(), Some(5));

// 链式操作
slice.push(6).push(7);
assert_eq!(slice.len(), 7);

// 集合运算
let a = AnySlice::new(vec![1, 2, 3]);
let b = vec![2, 3, 4];
assert_eq!(a.union(&b).to_vec(),        &vec![1, 2, 3, 4]);
assert_eq!(a.intersection(&b).to_vec(), &vec![2, 3]);
assert_eq!(a.difference(&b).to_vec(),   &vec![1]);
```

## API 一览

### 构造与数据访问

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `new(vec)` | `Vec<T> -> Self` | 从 `Vec<T>` 创建 | |
| `get_data(self)` | `-> Vec<T>` | 消耗自身，返回内部 `Vec<T>` | |
| `to_vec(&self)` | `-> &Vec<T>` | 返回内部 `Vec<T>` 的**引用** | |
| `set_data(&mut self, data)` | `Vec<T> -> &mut Self` | 替换内部数据 | |
| `copy(&self)` | `-> Self` | 深拷贝 | `T: Clone` |
| `clean(&mut self)` | `-> &mut Self` | 原地清空所有数据 | |
| `len(&self)` | `-> usize` | 元素个数 | |

### 元素操作

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `set_value(&mut self, index, value)` | `usize, T -> &mut Self` | 设置指定索引的值。**越界会 panic** | |
| `push(&mut self, value)` | `T -> &mut Self` | 追加单个元素，支持链式调用 | |
| `append(&mut self, values)` | `Vec<T> -> &mut Self` | 追加多个元素，支持链式调用 | |
| `filter(&mut self, predicate)` | `impl Fn(&T) -> bool` `-> &mut Self` | 原地过滤，仅保留满足条件的元素 | |
| `each(&mut self, func)` | `impl Fn(usize, &T) -> T` `-> &mut Self` | 对每个元素执行转换并原地更新，回调首参为索引 | |
| `sort(&mut self, func)` | `impl Fn(&T, &T) -> Ordering` `-> &mut Self` | 按自定义比较函数排序 | |
| `remove_by_index(&mut self, index)` | `&usize -> &mut Self` | **原地**移除，参数是引用；越界安全 | |
| `remove_by_indexes(&mut self, indexes)` | `&Vec<usize> -> &mut Self` | ⚠️ 当前无效，见「已知问题」 | |
| `remove_empty(&mut self)` | `-> &mut Self` | 移除所有默认值元素（如 `0`、`""`） | `T: PartialEq + Default + Clone` |

### 查询与判断

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `empty(&self)` | `-> bool` | 是否为空 | |
| `not_empty(&self)` | `-> bool` | 是否非空 | |
| `has(&self, values)` | `&Vec<T> -> bool` | **任意一个**元素命中即 `true` | `T: PartialEq` |
| `not_has(&self, values)` | `&Vec<T> -> bool` | `!has(...)`，即一个都不命中 | `T: PartialEq` |
| `len_without_empty(&self)` | `-> usize` | 非默认值元素个数 | `T: PartialEq + Default + Clone` |
| `all_empty(&self)` | `-> bool` | 是否所有元素都是默认值 | `T: PartialEq + Default + Clone` |
| `any_empty(&self)` | `-> bool` | 是否存在默认值元素 | `T: PartialEq + Default + Clone` |
| `every(&self, func)` | `impl Fn(usize, &T) -> bool` `-> bool` | 是否所有元素都满足条件 | |

### 元素获取

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `get_value_by_index(&self, index)` | `usize -> Option<T>` | 返回**克隆值**；越界返回 `None` | `T: Clone` |
| `get_value_ptr(&self, index)` | `usize -> Option<*const T>` | 获取指定索引的原始指针 | |
| `get_value_default(&self, index, default)` | `usize, T -> T` | 越界时返回给定默认值 | `T: Clone` |
| `get_values(&self, indexes)` | `&[usize] -> Vec<T>` | 批量获取；越界索引被跳过 | `T: Clone` |
| `get_values_by_slicer(&self, slicer)` | `&AnySlice<usize> -> Vec<T>` | 按索引切片批量获取 | `T: Clone` |
| `first(&self)` | `-> Option<T>` | 第一个元素的克隆值 | `T: Clone` |
| `last(&self)` | `-> Option<T>` | 最后一个元素的克隆值 | `T: Clone` |

### 索引查找

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `get_indexes(&self)` | `-> Vec<usize>` | 全部索引 `[0, 1, ..., n-1]` | |
| `get_index_by_value(&self, value)` | `&T -> Option<usize>` | 按值查找第一个匹配的索引 | `T: PartialEq` |
| `get_indexes_by_values(&self, values)` | `&Vec<T> -> Vec<usize>` | 批量查找。⚠️ **任一值不存在会 panic**（内部 `unwrap`） | `T: PartialEq + Clone` |

### 集合运算

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `union(&self, other)` | `&Vec<T> -> AnySlice<T>` | 并集，保留 `self` 顺序后追加新元素 | `T: PartialEq + Clone` |
| `union_slicer(&self, other)` | `&AnySlice<T> -> AnySlice<T>` | 并集 | `T: PartialEq + Clone` |
| `intersection(&self, other)` | `&Vec<T> -> AnySlice<T>` | 交集 | `T: PartialEq + Clone` |
| `intersection_slicer(&self, other)` | `&AnySlice<T> -> AnySlice<T>` | 交集 | `T: PartialEq + Clone` |
| `difference(&self, other)` | `&Vec<T> -> AnySlice<T>` | 差集（`self - other`） | `T: PartialEq + Clone` |
| `difference_slicer(&self, other)` | `&AnySlice<T> -> AnySlice<T>` | 差集 | `T: PartialEq + Clone` |

### 转换与工具

| 方法 | 签名 | 说明 | 约束 |
|------|------|------|------|
| `chunk(&self, size)` | `usize -> Vec<Vec<T>>` | 按指定大小分块。**`size` 为 0 会 panic** | `T: Clone` |
| `pluck<DST>(&self, func)` | `impl Fn(&T) -> DST` `-> Vec<DST>` | 映射每个元素并收集为 `Vec` | |
| `shuffle(&self)` | `-> AnySlice<T>` | ⚠️ 当前无效，见「已知问题」 | `T: Clone` |
| `shuffle_self(&mut self)` | `-> &mut Self` | 原地随机打乱，**可用** | |
| `to_string(&self, sep)` | `Option<&str> -> String` | 转为字符串，默认分隔符 `,` | `T: Display` |

## 已知问题

### `remove_by_indexes` 不生效

```rust
// src/any_slices/app.rs:347
pub fn remove_by_indexes(&mut self, indexes: &Vec<usize>) -> &mut Self {
    let _ = indexes.iter().map(|index| self.data.remove(*index)); // map 惰性，立即被丢弃
    self
}
```

`Iterator::map` 是惰性适配器，`let _ =` 使其在未消费时立即析构，因此没有任何元素被移除。该方法目前等价于空操作。

这同时导致依赖它的 `AnyMap::filter`、`AnyMap::remove_by_keys`、`AnyMap::remove_by_values`、`AnyMap::remove_empty` 全部失效，详见 [any_maps/README.md](../any_maps/README.md)。

临时替代写法：

```rust
let mut slice = AnySlice::new(vec![1, 2, 3, 4, 5]);
let remove = vec![0usize, 2];

// 用 filter 按下标保留，代替 remove_by_indexes
let kept: Vec<i32> = slice.to_vec()
    .iter()
    .enumerate()
    .filter(|(i, _)| !remove.contains(i))
    .map(|(_, v)| *v)
    .collect();
slice.set_data(kept);
assert_eq!(slice.to_vec(), &vec![2, 4, 5]);
```

### `shuffle` 返回未打乱的副本

`shuffle` 对一个临时克隆执行打乱后将其丢弃，返回的仍是原顺序数据。需要打乱请用 `shuffle_self`。

## 链式调用示例

### 排序 + 过滤 + 转字符串

```rust
use nothings::any_slices::app::AnySlice;

let mut slice = AnySlice::new(vec![5, 3, 1, 4, 2]);

slice.sort(|a, b| a.cmp(b))
     .filter(|x| *x > 2);

println!("{}", slice.to_string(Some(" -> ")));
// 输出: 3 -> 4 -> 5
```

### 集合运算

```rust
use nothings::any_slices::app::AnySlice;

let a = AnySlice::new(vec![1, 2, 3, 4]);
let b = AnySlice::new(vec![3, 4, 5, 6]);

let result = a.union_slicer(&b)
              .difference_slicer(&AnySlice::new(vec![5]));

println!("{:?}", result.to_vec());
// 输出: [1, 2, 3, 4, 6]
```

### pluck 提取字段

```rust
use nothings::any_slices::app::AnySlice;

struct User { name: String, age: u8 }

let users = AnySlice::new(vec![
    User { name: "Alice".into(), age: 30 },
    User { name: "Bob".into(), age: 25 },
]);

let names: Vec<String> = users.pluck(|u| u.name.clone());
// ["Alice", "Bob"]
let ages: Vec<u8> = users.pluck(|u| u.age);
// [30, 25]
```

### each 原地转换

```rust
use nothings::any_slices::app::AnySlice;

let mut slice = AnySlice::new(vec![1, 2, 3]);

// 回调首参是索引
slice.each(|idx, item| item * 10 + idx as i32);

assert_eq!(slice.to_vec(), &vec![10, 21, 32]);
```

## 备注

- `app.rs` 中声明了 `AnySliceTrait`，但 `AnySlice` 并未实现它，仅作为接口说明存在。请勿在代码中 `use` 该 trait。
- 集合运算（`union` / `intersection` / `difference`）均为 O(n·m) 线性查找，不适合大数据量。
