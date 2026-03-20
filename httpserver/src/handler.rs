use http::httprequest::HttpRequest;
use http::httpresponse::HttpResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{env, fs};

// Handler：最后真正在对不同的request请求进行处理的地方，handler是实现了共同的方法trait
pub trait Handler {
    // 不同的Handler需要特化实现不同的handle，但都返回同样格式的HttpResponse
    fn handle(request: &HttpRequest) -> HttpResponse<'_>;
    // load_file是不同的Handler都需要的函数，用于得到HttpResponse中的body部分
    fn load_file(file_name: &str) -> Option<String> {
        // 得到在项目根目录下的public子目录
        // env!("CARGO_MANIFEST_DIR")编译期宏，获取  Cargo.toml  所在目录的绝对路径
        let default_path = format!("{}/public", env!("CARGO_MANIFEST_DIR"));
        //  env::var("PUBLIC_PATH")运行时读取环境变量  PUBLIC_PATH，返回Result
        // 通过unwrap_or解包，成功返回成功值，失败返回default_path
        let public_path = env::var("PUBLIC_PATH").unwrap_or(default_path);
        // 拼接完整文件路径：基础目录（来自上一步）拼接上 用户请求的具体文件名
        let full_path = format!("{}/{}", public_path, file_name);
        // 可能失败的情况： 文件不存在、无权限、不是有效UTF-8文本
        let contents = fs::read_to_string(full_path);
        // 成功返回转换后的Some，失败返回None
        contents.ok()
    }
}
// 表示用于处理静态页面的Handler
pub struct StaticPageHandler;
// 404的Handler
pub struct PageNotFoundHandler;

// Web请求的Handler
pub struct WebServiceHandler;
// OrderStatus是对接WebServiceHandler中会读取的json文件，用于反序列化
#[derive(Deserialize, Serialize)]
pub struct OrderStatus {
    order_id: i32,
    order_date: String,
    order_status: String,
}

impl Handler for StaticPageHandler {
    fn handle(request: &HttpRequest) -> HttpResponse<'_> {
        let http::httprequest::HttpReSource::Path(path) = &request.resource();
        // 静态页面请求直接读取/后目标
        let route: Vec<&str> = path.split('/').collect();
        // 不同的请求参数有不同的返回值
        match route[1] {
            "" => HttpResponse::new("200", None, Self::load_file("index.html")),
            "health" => HttpResponse::new("200", None, Self::load_file("health.html")),
            path => match Self::load_file(path) {
                Some(contents) => {
                    // 请求不同的文件需要在httpresponse的响应头中添加不同的Type标识
                    let mut map: HashMap<&str, &str> = HashMap::new();
                    if path.ends_with(".css") {
                        map.insert("Content-Type", "text/css");
                    } else if path.ends_with(".js") {
                        map.insert("Content-Type", "application/javascript");
                    } else {
                        map.insert("Content-Type", "text/html");
                    }
                    HttpResponse::new("200", Some(map), Some(contents))
                }
                None => HttpResponse::new("404", None, Self::load_file("404.html")),
            },
        }
    }
}
impl Handler for PageNotFoundHandler {
    fn handle(_request: &HttpRequest) -> HttpResponse<'_> {
        HttpResponse::new("404", None, Self::load_file("404.html"))
    }
}

impl WebServiceHandler {
    fn load_json() -> Vec<OrderStatus> {
        // 处理对应路径
        let default_path = format!("{}/data", env!("CARGO_MANIFEST_DIR"));
        let data_path = env::var("DATA_PATH").unwrap_or(default_path);
        let full_path = format!("{}/{}", data_path, "orders.json");
        let json_contents = fs::read_to_string(full_path);
        let orders: Vec<OrderStatus> =
            serde_json::from_str(json_contents.unwrap().as_str()).unwrap();
        orders
    }
}
impl Handler for WebServiceHandler {
    fn handle(request: &HttpRequest) -> HttpResponse<'_> {
        let http::httprequest::HttpReSource::Path(path) = &request.resource();
        let route: Vec<&str> = path.split('/').collect();
        match route[2] {
            "shipping" if route.len() > 2 && route[3] == "orders" => {
                let body = Some(serde_json::to_string(&Self::load_json()).unwrap());
                let mut headers: HashMap<&str, &str> = HashMap::new();
                headers.insert("Content-Type", "application/json");
                HttpResponse::new("200", Some(headers), body)
            }
            _ => HttpResponse::new("404", None, Self::load_file("404.html")),
        }
    }
}
