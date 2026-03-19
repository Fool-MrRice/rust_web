// use anyhow::Result;
use std::collections::HashMap;

// 需要判断是get还是post故需要PartialEq
#[derive(Debug, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Uninitialized,
}
impl From<&str> for HttpMethod {
    fn from(s: &str) -> HttpMethod {
        match s {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            _ => HttpMethod::Uninitialized,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum HttpVersion {
    V1_1,
    V2_0,
    Uninitialized,
}
impl From<&str> for HttpVersion {
    fn from(s: &str) -> HttpVersion {
        match s {
            "HTTP/1.1" => HttpVersion::V1_1,
            _ => HttpVersion::Uninitialized,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HttpReSource {
    Path(String),
}
//仅仅解析故不需要PartialEq来比较是否是相同的Http请求
#[derive(Debug)]
pub struct HttpRequest {
    // 请求行（METHOD 请求目标 HTTP/版本）
    // 请求头（可选）
    // 空行
    // 请求体（可选）
    method: HttpMethod,
    version: HttpVersion,
    resource: HttpReSource,
    head: Option<HashMap<String, String>>,
    body: Option<String>,
}
// 用String是因为需要长时间使用，故需要所有权
impl From<String> for HttpRequest {
    fn from(req: String) -> Self {
        //  先初始化HttpRequest
        let mut parsed_method = HttpMethod::Uninitialized;
        let mut parsed_version = HttpVersion::Uninitialized;
        let mut parsed_resource = HttpReSource::Path("".to_string());
        let mut parsed_head = HashMap::new();
        let mut parsed_body = "".to_string();

        let mut request_line_parsed = false;
        // 标记是否在 body 部分
        let mut in_body = false;

        for line in req.lines() {
            // 关键修复1：处理 \r，Windows 换行是 \r\n，lines() 会保留 \r
            let line = line.trim_end_matches('\r');

            if !request_line_parsed && line.contains("HTTP") {
                let (method, resource, version) = process_req_line(line);
                parsed_method = method;
                parsed_version = version;
                parsed_resource = resource;
                request_line_parsed = true;
            } else if in_body {
                // 关键修复2：已经在 body 部分，累积所有行
                if !parsed_body.is_empty() {
                    parsed_body.push('\n');
                }
                parsed_body.push_str(line);
            } else if line.contains(':') {
                // 关键修复3：使用 splitn(2, ':') 只分割第一个冒号
                let (key, value) = process_req_head(line);
                parsed_head.insert(key, value);
            } else if line.is_empty() {
                // 关键修复4：空行表示 header 结束，接下来是 body
                in_body = true;
            }
            // 其他情况（如 header 前的空行）忽略
        }
        HttpRequest {
            method: parsed_method,
            version: parsed_version,
            resource: parsed_resource,
            head: Some(parsed_head),
            body: Some(parsed_body),
        }
    }
}

fn process_req_head(line: &str) -> (String, String) {
    let mut parts = line.splitn(2, ':');
    let key = parts.next().unwrap().trim().to_string();
    let value = parts.next().unwrap().trim().to_string();
    (key, value)
}

fn process_req_line(line: &str) -> (HttpMethod, HttpReSource, HttpVersion) {
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap();
    let resource = parts.next().unwrap();
    let version = parts.next().unwrap();
    (
        method.into(),
        HttpReSource::Path(resource.into()),
        version.into()
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    // ========== HttpMethod 测试 ==========

    #[test]
    fn test_http_method_from_get() {
        assert_eq!(HttpMethod::from("GET"), HttpMethod::Get);
    }

    #[test]
    fn test_http_method_from_post() {
        assert_eq!(HttpMethod::from("POST"), HttpMethod::Post);
    }

    #[test]
    fn test_http_method_from_unknown() {
        assert_eq!(HttpMethod::from("DELETE"), HttpMethod::Uninitialized);
        assert_eq!(HttpMethod::from("PUT"), HttpMethod::Uninitialized);
        assert_eq!(HttpMethod::from(""), HttpMethod::Uninitialized);
    }

    // ========== HttpVersion 测试 ==========

    #[test]
    fn test_http_version_from_v1_1() {
        assert_eq!(HttpVersion::from("HTTP/1.1"), HttpVersion::V1_1);
    }

    #[test]
    fn test_http_version_from_unknown() {
        assert_eq!(HttpVersion::from("HTTP/2.0"), HttpVersion::Uninitialized);
        assert_eq!(HttpVersion::from("HTTP/1.0"), HttpVersion::Uninitialized);
        assert_eq!(HttpVersion::from(""), HttpVersion::Uninitialized);
    }

    // ========== HttpRequest 解析测试 ==========

    /// 辅助函数：创建一个简单的 GET 请求字符串
    fn make_get_request() -> String {
        "GET /index.html HTTP/1.1\r\n\
         Host: localhost:8080\r\n\
         User-Agent: curl/7.64.1\r\n\
         Accept: */*\r\n\
         \r\n"
            .to_string()
    }

    /// 辅助函数：创建一个 POST 请求字符串
    fn make_post_request() -> String {
        "POST /api/users HTTP/1.1\r\n\
         Host: api.example.com\r\n\
         Content-Type: application/json\r\n\
         Content-Length: 27\r\n\
         \r\n\
         {\"name\":\"tom\",\"age\":25}"
            .to_string()
    }

    #[test]
    fn test_parse_simple_get_request() {
        let request = HttpRequest::from(make_get_request());
        let head = request.head.unwrap();
        assert_eq!(request.method, HttpMethod::Get);
        assert_eq!(request.version, HttpVersion::V1_1);

        // 检查 resource
        match request.resource {
            HttpReSource::Path(path) => assert_eq!(path, "/index.html"),
        }

        // 检查 headers
        assert_eq!(head.get("Host"), Some(&"localhost:8080".to_string()));
        assert_eq!(head.get("User-Agent"), Some(&"curl/7.64.1".to_string()));
        assert_eq!(head.get("Accept"), Some(&"*/*".to_string()));

        // GET 请求没有 body
        assert_eq!(request.body.unwrap(), "");
    }

    #[test]
    fn test_parse_post_request_with_body() {
        let request = HttpRequest::from(make_post_request());
        let head = request.head.unwrap();
        assert_eq!(request.method, HttpMethod::Post);

        match &request.resource {
            HttpReSource::Path(path) => assert_eq!(path, "/api/users"),
        }

        assert_eq!(head.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(request.body.unwrap(), "{\"name\":\"tom\",\"age\":25}");
    }

    #[test]
    fn test_parse_request_with_empty_headers() {
        let req_str = "GET / HTTP/1.1\r\n\r\n".to_string();
        let request = HttpRequest::from(req_str);
        let head = request.head.unwrap();
        assert_eq!(request.method, HttpMethod::Get);
        assert!(head.is_empty());
        assert_eq!(request.body.unwrap(), "");
    }

    #[test]
    fn test_parse_request_with_multiple_colons_in_header() {
        // Header value 中可能包含冒号，比如时间
        let req_str = "GET / HTTP/1.1\r\n\
                       Date: Mon, 23 May 2022 22:38:34 GMT\r\n\
                       \r\n"
            .to_string();

        let request = HttpRequest::from(req_str);
        let head = request.head.unwrap();
        // 注意：你的实现只用第一个冒号分割，后面的会保留
        assert_eq!(
            head.get("Date"),
            Some(&"Mon, 23 May 2022 22:38:34 GMT".to_string())
        );
    }

    // ========== 边界情况测试 ==========

    #[test]
    fn test_parse_empty_string() {
        let request = HttpRequest::from("".to_string());
        let head = request.head.unwrap();
        assert_eq!(request.method, HttpMethod::Uninitialized);
        assert_eq!(request.version, HttpVersion::Uninitialized);
        assert!(head.is_empty());
    }

    #[test]
    fn test_parse_only_whitespace() {
        let request = HttpRequest::from("   \n   \n   ".to_string());

        assert_eq!(request.method, HttpMethod::Uninitialized);
        assert_eq!(request.body.unwrap(), "");
    }


    // ========== 多行 body 测试 ==========

    #[test]
    fn test_parse_multiline_body() {
        // 注意：你的实现用 lines() 会丢失换行符
        // 而且空行判断逻辑有问题，这个测试会暴露问题

        let req_str = "POST /upload HTTP/1.1\r\n\
                       Content-Length: 20\r\n\
                       \r\n\
                       line1\r\n\
                       line2\r\n\
                       line3"
            .to_string();

        let request = HttpRequest::from(req_str);

        // 当前实现只会保留最后一行 "line3"
        // 因为 lines() 会跳过空行，且每次赋值覆盖 parsed_body
        assert_eq!(request.body.unwrap(), "line1\nline2\nline3");
    }
}
