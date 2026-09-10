# HttpResponse — 统一响应结构

`http_responses` 模块提供一个可序列化的泛型响应包装 `HttpResponse<T>`，用于把业务结果统一成 `{ code, msg, content }` 的 JSON 形状。

```rust
use nothings::http_responses::HttpResponse;
```

模块内没有任何其它类型在使用它，是纯粹给调用方用的对外结构。

## 结构定义

```rust
#[derive(Serialize)]
pub struct HttpResponse<T> {
    pub code: u32,
    pub msg: String,
    pub content: Option<T>,
    #[serde(skip)]
    pub errors: Vec<std::fmt::Error>,
}
```

四个字段**全部是 `pub`**，可以直接读写。

序列化输出只有三个键——`errors` 被 `#[serde(skip)]` 排除：

```json
{"code":200,"msg":"OK","content":42}
```

`content` 为 `None` 时会输出 `"content":null`，**不会**被省略（没有 `skip_serializing_if`）。

只派生了 `Serialize`。序列化 `content` 要求 `T: Serialize`，因此下游需要自己依赖 `serde`：

```rust
use serde::Serialize;
use nothings::http_responses::HttpResponse;

#[derive(Serialize)]
struct User { id: u32, name: String }

let r = HttpResponse::ok(None).content(User { id: 1, name: "alice".into() });
println!("{}", serde_json::to_string(&r).unwrap());
// {"code":200,"msg":"OK","content":{"id":1,"name":"alice"}}
```

## 构造器

全部是关联函数，返回 `HttpResponse<T>`：

| 构造器 | 签名 | code | 默认 msg |
|--------|------|------|----------|
| `ok(msg)` | `Option<&str> -> Self` | 200 | `OK` |
| `created(msg)` | `Option<&str> -> Self` | 201 | `创建成功` |
| `updated(msg)` | `Option<String> -> Self` | 202 | `编辑成功` |
| `deleted(msg)` | `Option<&str> -> Self` | 204 | `删除成功` |
| `bad_request(msg)` | `&str -> Self` | 400 | 无默认值，**必填** |
| `un_authorization(msg)` | `Option<&str> -> Self` | 401 | `未授权` |
| `internal_server_error(msg)` | `&str -> Self` | 500 | 无默认值，**必填** |

⚠️ **`updated` 的参数是 `Option<String>`，其余可选参数的构造器都是 `Option<&str>`**。这是接口不一致，写代码时容易踩：

```rust
use nothings::http_responses::HttpResponse;

let _: HttpResponse<()> = HttpResponse::ok(Some("fine"));            // &str
let _: HttpResponse<()> = HttpResponse::updated(Some("已更新".to_string()));  // String
```

## 链式构造器

| 方法 | 签名 | 说明 |
|------|------|------|
| `content(self, value)` | `T -> Self` | 填入响应体，消耗并返回自身 |
| `error(self, err)` | `std::fmt::Error -> Self` | 向 `errors` 追加一项（实际无用，见下） |

```rust
use nothings::http_responses::HttpResponse;

let ok: HttpResponse<i32> = HttpResponse::ok(None).content(42);
assert_eq!(ok.code, 200);
assert_eq!(ok.content, Some(42));

let err: HttpResponse<String> = HttpResponse::bad_request("参数错误").content("username".into());
assert_eq!(err.code, 400);
assert_eq!(err.msg, "参数错误");
```

`content` 与 `error` 都接收 `mut self` 并返回 `Self`，所以可以连续链式调用；但 `content` 只能调一次（第二次会覆盖）。

## 完整示例

```rust
use nothings::http_responses::HttpResponse;

fn login(user: Option<&str>) -> HttpResponse<String> {
    match user {
        Some(name) => HttpResponse::ok(None).content(format!("welcome, {name}")),
        None => HttpResponse::un_authorization(Some("请先登录")),
    }
}

let a = login(Some("alice"));
let b = login(None);

assert_eq!(a.code, 200);
assert_eq!(b.code, 401);
assert_eq!(b.msg, "请先登录");
assert!(b.content.is_none());

println!("{}", serde_json::to_string(&b).unwrap());
// {"code":401,"msg":"请先登录","content":null}
```

## 限制

### 1. 无法自定义状态码

私有的 `empty()` 是唯一的字段初始化入口，所有构造器都基于它。因此**只能产出上表那 7 个状态码**，需要 403 / 404 / 409 / 429 等只能绕过去：

```rust
use nothings::http_responses::HttpResponse;

// 借任意构造器起手，再直接改 pub 字段
let mut r: HttpResponse<()> = HttpResponse::bad_request("");
r.code = 404;
r.msg = "资源不存在".to_string();
assert_eq!(r.code, 404);
```

因为字段是 `pub` 的，这种写法可行，但语义上不优雅。

### 2. `errors` 字段实际上没有用处

`std::fmt::Error` 是个**不携带任何信息**的错误类型（它只表示「格式化失败」），既没有构造参数也没有 `Display` 内容可区分。加上 `#[serde(skip)]`，实测：

```rust
use nothings::http_responses::HttpResponse;

let r: HttpResponse<i32> = HttpResponse::ok(None).error(std::fmt::Error::default());
assert_eq!(r.errors.len(), 1);
// 序列化结果里完全没有 errors
assert_eq!(serde_json::to_string(&r).unwrap(), r#"{"code":200,"msg":"OK","content":null}"#);
```

即：能塞进去，但塞进去的东西不含信息，也永远序列化不出来。想返回校验错误明细，请把它们放进 `content`（例如 `HttpResponse<Vec<String>>`），或配合 [`validations`](../validations/README.md) 模块自行组装。

### 3. 没有 `Debug` / `Clone` / `PartialEq` / `Deserialize`

```rust
// 编译失败：`HttpResponse<i32>` doesn't implement `Debug`
// println!("{:?}", HttpResponse::<i32>::ok(None));
```

后果：

- 不能用 `{:?}` 打印，调试只能逐字段输出或先 `serde_json::to_string`
- 不能 `assert_eq!` 比较两个响应，测试里得比对 `code` / `msg` / `content`
- 不能克隆，需要多份时只能重新构造
- 不能反序列化，**只适合做服务端出参，不适合做客户端入参**

调试时的替代写法：

```rust
use nothings::http_responses::HttpResponse;

let r: HttpResponse<i32> = HttpResponse::ok(None).content(1);
println!("{}", serde_json::to_string(&r).unwrap());
```

### 4. 状态码语义与 HTTP 标准有出入

| code | 本模块含义 | HTTP 标准含义 |
|------|-----------|--------------|
| 202 | 编辑成功 | Accepted（已接受，尚未处理完成） |
| 204 | 删除成功 | No Content（**响应体必须为空**） |

204 在标准里明确不允许带响应体，而本模块的 `deleted()` 仍可以 `.content(...)`。如果这套 code 会直接映射成真实 HTTP 状态码，需要自己留意；如果只是业务约定的返回码（很多国内接口就是这么用的），则无影响。

## 备注

- 模块还定义了 `pub type HTTPResponseAttr<T> = Box<dyn Fn(&HttpResponse<T>)>;`，但**整个代码库没有任何地方使用它**，可以忽略
- 模块路径是 `nothings::http_responses`（复数、带下划线），结构体是单数 `HttpResponse`
- 没有 `new()` / `Default`，必须走具名构造器
