use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// 从 #[validator(...)] 属性中解析键值对
/// 自定义解析器，正确处理字符串值内的逗号
fn parse_validator_attrs(tokens: &proc_macro2::TokenStream) -> Vec<(String, String)> {
    let token_str = tokens.to_string();
    let chars: Vec<char> = token_str.chars().collect();
    let len = chars.len();
    let mut result = Vec::new();
    let mut i = 0;

    while i < len {
        // 跳过空白和逗号
        while i < len && (chars[i].is_whitespace() || chars[i] == ',') {
            i += 1;
        }
        if i >= len {
            break;
        }

        // 读取 key（到 '=' 或空白为止）
        let key_start = i;
        while i < len && chars[i] != '=' && !chars[i].is_whitespace() {
            i += 1;
        }
        let key = chars[key_start..i].iter().collect::<String>().trim().to_string();
        if key.is_empty() {
            break;
        }

        // 跳过空白
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        // 检查是否有 '='（表示有值）
        if i < len && chars[i] == '=' {
            i += 1; // 跳过 '='
            // 跳过空白
            while i < len && chars[i].is_whitespace() {
                i += 1;
            }

            if i < len && chars[i] == '"' {
                // 解析字符串值（正确处理内部逗号）
                i += 1; // 跳过开始引号
                let val_start = i;
                while i < len && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 2; // 跳过转义字符
                    } else {
                        i += 1;
                    }
                }
                let value: String = chars[val_start..i].iter().collect();
                if i < len {
                    i += 1; // 跳过结束引号
                }
                result.push((key, value));
            }
        } else {
            // 无值的标志（如 nested）
            result.push((key, String::new()));
        }
    }

    result
}

/// 派生宏：解析结构体字段上的 #[validator(...)] 属性
/// 
/// 使用示例：
/// ```ignore
/// #[derive(Nothings)]
/// struct MyStruct {
///     #[validator(rule = "min:`>10`;max:`<=100`;size:`=20`" name = "年龄" kind = "usize")]
///     age: usize,
///     
///     // 嵌套结构体，递归解析
///     #[validator(nested)]
///     address: Address,
/// }
/// ```
/// 
/// 规则语法：
/// - 规则之间用 `;` 分隔
/// - 格式：`key:`操作符+值``  如 `min:`>10``、`size:`=20``
/// - 支持的操作符：`>`, `<`, `>=`, `<=`, `=`, `!=`
/// - `ex` 规则使用 `:` 后跟函数列表：`ex:`fn1,fn2``
/// - `in` 规则支持 `=` (在列表中) 和 `!=` (不在列表中)：`in:`=a,b,c``、`in:`!=x,y,z``
/// 
/// 嵌套语法：
/// - 使用 `nested` 标记嵌套结构体字段
/// - 嵌套字段名会自动添加前缀（如 `address.city`）
#[proc_macro_derive(Nothings, attributes(validator))]
pub fn derive_nothings(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields_extraction = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                let field_extractions = fields.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_name_str = field_name.to_string();
                    
                    // 获取字段类型
                    let field_type = &field.ty;
                    let field_type_str = quote!(#field_type).to_string();
                    
                    // 检测是否为 Option<T> 类型
                    let is_option = if let syn::Type::Path(type_path) = field_type {
                        type_path.path.segments.last()
                            .map(|seg| seg.ident == "Option")
                            .unwrap_or(false)
                    } else {
                        false
                    };
                    
                    // 解析 #[validator(...)] 属性（使用自定义解析器，正确处理字符串内逗号）
                    let mut rules_str = String::new();
                    let mut name_value = String::new();
                    let mut kind_value = String::new();
                    let mut has_rule = false;
                    let mut has_name = false;
                    let mut is_nested = false;
                    
                    for attr in &field.attrs {
                        if attr.path().is_ident("validator") {
                            if let Ok(meta_list) = attr.meta.require_list() {
                                let attrs = parse_validator_attrs(&meta_list.tokens);
                                for (key, value) in attrs {
                                    match key.as_str() {
                                        "nested" => is_nested = true,
                                        "rule" => {
                                            rules_str = value;
                                            has_rule = true;
                                        }
                                        "name" => {
                                            name_value = value;
                                            has_name = true;
                                        }
                                        "kind" => kind_value = value,
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    
                    // 处理嵌套结构体
                    if is_nested {
                        return quote! {
                            {
                                let nested_fields = ::nothings::validations::validator_struct_parser::NothingsValidatorStructParser::parse_fields(&self.#field_name);
                                for mut f in nested_fields {
                                    f.name = format!("{}.{}", #field_name_str, f.name);
                                    fields.push(f);
                                }
                            }
                        };
                    }
                    
                    // 如果 rule 和 name 没有同时提供，跳过该字段
                    if !has_rule || !has_name {
                        return quote! {};
                    }
                    
                    // 如果没有指定 kind，使用字段类型
                    if kind_value.is_empty() {
                        kind_value = field_type_str.clone();
                    }
                    
                    // 解析 rule 字符串，将每条规则转换为 Vec 元素
                    let rules: Vec<String> = if rules_str.is_empty() {
                        vec![]
                    } else {
                        rules_str.split(';')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    };

                    if is_option {
                        let rules_clone = rules.clone();
                        let name_clone = name_value.clone();
                        let kind_clone = kind_value.clone();
                        quote! {
                            match &self.#field_name {
                                Some(val) => {
                                    fields.push(::nothings::validations::field::Field {
                                        origin: format!("{}", val),
                                        name: #name_clone.to_string(),
                                        kind: #kind_clone.to_string(),
                                        rules: vec![#(#rules_clone.to_string()),*],
                                        is_option: true,
                                    });
                                }
                                None => {
                                    let rules = vec![#(#rules.to_string()),*];
                                    if rules.contains(&"required".to_string()) {
                                        fields.push(::nothings::validations::field::Field {
                                            origin: String::new(),
                                            name: #name_value.to_string(),
                                            kind: #kind_value.to_string(),
                                            rules,
                                            is_option: true,
                                        });
                                    }
                                }
                            }
                        }
                    } else {
                        quote! {
                            fields.push(::nothings::validations::field::Field {
                                origin: format!("{}", self.#field_name),
                                name: #name_value.to_string(),
                                kind: #kind_value.to_string(),
                                rules: vec![#(#rules.to_string()),*],
                                is_option: false,
                            });
                        }
                    }
                });

                quote! {
                    let mut fields: Vec<::nothings::validations::field::Field> = Vec::new();
                    #(#field_extractions)*
                    fields
                }
            }
            _ => {
                quote! {
                    compile_error!("Nothings 只支持命名字段的结构体");
                    vec![]
                }
            }
        },
        _ => {
            quote! {
                compile_error!("Nothings 只能用于结构体");
                vec![]
            }
        }
    };

    let expanded = quote! {
        impl #impl_generics ::nothings::validations::validator_struct_parser::NothingsValidatorStructParser for #name #ty_generics #where_clause {
            fn parse_fields(&self) -> Vec<::nothings::validations::field::Field> {
                #fields_extraction
            }
        }
    };

    TokenStream::from(expanded)
}
