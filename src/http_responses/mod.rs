use std::fmt::Error;
use serde::Serialize;

#[derive(Serialize)]
pub struct HttpResponse<T> {
    pub code: u32,
    pub msg: String,
    pub content: Option<T>,
    #[serde(skip)]
    pub errors: Vec<Error>,
}

pub type HTTPResponseAttr<T> = Box<dyn Fn(&HttpResponse<T>)>;

impl<T> HttpResponse<T> {
    fn empty() -> HttpResponse<T> {
        HttpResponse {
            code: 0,
            msg: "".to_string(),
            content: None,
            errors: vec!(),
        }
    }
    
    pub fn ok(msg: Option<String>) -> HttpResponse<T> {
        HttpResponse {
            code: 200,
            msg: msg.unwrap_or_else(|| "OK".to_string()),
            ..HttpResponse::empty()
        }
    }
    
    pub fn created(msg: Option<String>) -> HttpResponse<T> {
        HttpResponse {
            code: 201,
            msg: msg.unwrap_or_else(|| "创建成功".to_string()),
            ..HttpResponse::empty()
        }
    }
    
    pub fn updated(msg: Option<String>) -> HttpResponse<T> {
        HttpResponse {
            code: 202,
            msg: msg.unwrap_or_else(|| "编辑成功".to_string()),
            ..HttpResponse::empty()
        }
    }
    
    pub fn deleted(msg: Option<String>) -> HttpResponse<T> {
        HttpResponse {
            code: 204,
            msg: msg.unwrap_or_else(|| "删除成功".to_string()),
            ..HttpResponse::empty()
        }
    }
    
    pub fn bad_request(msg: String) -> HttpResponse<T> {
        HttpResponse {
            code: 400,
            msg,
            ..HttpResponse::empty()
        }
    }
    
    pub fn un_authorization(msg: Option<String>) -> HttpResponse<T> {
        HttpResponse {
            code: 401,
            msg: msg.unwrap_or_else(|| "未授权".to_string()),
            ..HttpResponse::empty()
        }
    }
    
    pub fn internal_server_error(msg: String) -> HttpResponse<T> {
        HttpResponse {
            code: 500,
            msg,
            ..HttpResponse::empty()
        }
    }
    
    pub fn content(mut self, content: T) -> HttpResponse<T>{
        self.content = Some(content);
        self
    }
    
    pub fn error(mut self, error: Error) -> HttpResponse<T> {
        self.errors.push(error);
        self
    }
}

