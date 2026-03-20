use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;

#[derive(Debug, PartialEq, Clone)]
pub struct HttpResponse<'a> {
    // 状态行 （HTTP/版本 状态码 状态描述）
    // 响应头（可选）
    // 空行
    // 响应体（可选）
    version: &'a str,
    status_code: &'a str,
    status_text: &'a str,
    head: Option<HashMap<&'a str, &'a str>>,
    body: Option<String>,
}
// Default trait
impl<'a> Default for HttpResponse<'a> {
    fn default() -> Self {
        Self {
            version: "HTTP/1.1".into(),
            status_code: "200".into(),
            status_text: "OK".into(),
            head: None,
            body: None,
        }
    }
}
// new()
impl<'a> HttpResponse<'a> {
    pub fn new(status_code: &'a str, headers: Option<HashMap<&'a str, &'a str>>, body: Option<String>) -> HttpResponse<'a> {
        // 先声明
        let mut response: HttpResponse<'a> = HttpResponse::default();
        // 初始化
        if status_code != "200" {
            response.status_code = status_code.into();
        }
        response.status_text = match response.status_code {
            "200" => "OK".into(),
            "400" => "Bad Request".into(),
            "404" => "Not Found".into(),
            "500" => "Internal Server Error".into(),
            _ => "ERROR".into()
        };
        response.head = match headers {
            Some(h) => Some(h),
            None => {
                let mut h = HashMap::new();
                h.insert("Content-Type", "text/html; charset=utf-8".into());
                Some(h)
            }
        };
        response.body = body;
        response
    }
    // send_response()将原始字节通过TCP传递
    pub fn send_response(&self, write_stream: &mut impl Write) -> Result<()> {
        let res = self.clone();
        let response_string: String = String::from(res);
        let _ = write!(write_stream, "{}", response_string)?;
        Ok(())
    }
    // getter

    pub fn version(&self) -> &'a str {
        self.version
    }

    pub fn status_code(&self) -> &'a str {
        self.status_code
    }

    pub fn status_text(&self) -> &'a str {
        self.status_text
    }

    pub fn head(&self) -> String {
        let head: HashMap<&str, &str> = self.head.clone().unwrap();
        let mut head_string = String::new();
        for (k, v) in head.iter() {
            head_string.push_str(&format!("{}:{}\r\n", k, v));
        }
        head_string
    }

    pub fn body(&self) -> String {
        match &self.body {
            Some(b) => String::from(b),
            None => String::new()
        }
    }
}
// From 能够将HttpResponse转化为String
impl<'a> From<HttpResponse<'a>> for String {
    fn from(res: HttpResponse) -> String {
        let res_clone = res.clone();
        format!(
            "{} {} {}\r\n{}Content-Length: {}\r\n\r\n{}",
            res_clone.version(), res_clone.status_code(), res_clone.status_text(),
            res_clone.head(), res_clone.body().len(), res_clone.body()
        )
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // ========== 基础构造测试 ==========

    #[test]
    fn test_default_response() {
        let res = HttpResponse::default();

        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), "200");
        assert_eq!(res.status_text(), "OK");
        assert_eq!(res.body(), "");

        // head 应该包含默认 Content-Type
        let head_str = res.head();
        assert!(head_str.contains("Content-Type:text/html; charset=utf-8"));
    }

    #[test]
    fn test_new_with_default_200() {
        let res = HttpResponse::new("200", None, None);

        assert_eq!(res.status_code(), "200");
        assert_eq!(res.status_text(), "OK");
    }

    // ========== 状态码测试 ==========

    #[test]
    fn test_status_400() {
        let res = HttpResponse::new("400", None, None);
        assert_eq!(res.status_code(), "400");
        assert_eq!(res.status_text(), "Bad Request");
    }

    #[test]
    fn test_status_404() {
        let res = HttpResponse::new("404", None, None);
        assert_eq!(res.status_code(), "404");
        assert_eq!(res.status_text(), "Not Found");
    }

    #[test]
    fn test_status_500() {
        let res = HttpResponse::new("500", None, None);
        assert_eq!(res.status_code(), "500");
        assert_eq!(res.status_text(), "Internal Server Error");
    }

    #[test]
    fn test_status_unknown() {
        let res = HttpResponse::new("418", None, None);
        assert_eq!(res.status_code(), "418");
        assert_eq!(res.status_text(), "ERROR");
    }

    // ========== 自定义 Headers 测试 ==========

    #[test]
    fn test_custom_headers() {
        let mut headers = HashMap::new();
        headers.insert("X-Custom-Header", "custom-value");
        headers.insert("Authorization", "Bearer token123");

        let res = HttpResponse::new("200", Some(headers), None);
        let head_str = res.head();

        assert!(head_str.contains("X-Custom-Header:custom-value"));
        assert!(head_str.contains("Authorization:Bearer token123"));
    }

    #[test]
    fn test_empty_headers_become_default() {
        // 传入 None 应该得到默认 Content-Type
        let res = HttpResponse::new("200", None, None);
        let head_str = res.head();
        assert!(head_str.contains("Content-Type:text/html; charset=utf-8"));
    }

    // ========== Body 测试 ==========

    #[test]
    fn test_with_body() {
        let body = "<html><body>Hello</body></html>".to_string();
        let res = HttpResponse::new("200", None, Some(body.clone()));

        assert_eq!(res.body(), body);
    }

    #[test]
    fn test_empty_body() {
        let res = HttpResponse::new("200", None, None);
        assert_eq!(res.body(), "");
    }

    // ========== String 转换测试（From trait） ==========

    #[test]
    fn test_into_string_basic() {
        let res = HttpResponse::default();
        let s: String = res.into();

        // 验证格式：HTTP/1.1 200 OK\r\n...
        assert!(s.starts_with("HTTP/1.1 200 OK\r\n"));
        assert!(s.contains("Content-Type:text/html; charset=utf-8"));
        assert!(s.contains("Content-Length: 0"));
        assert!(s.ends_with("\r\n\r\n"));  // 空body也以空行结束
    }

    #[test]
    fn test_into_string_with_body() {
        let body = "Hello, World!".to_string();
        let res = HttpResponse::new("200", None, Some(body));
        let s: String = res.into();

        // 验证 Content-Length 正确计算
        assert!(s.contains("Content-Length: 13"));
        assert!(s.ends_with("Hello, World!"));
    }

    #[test]
    fn test_into_string_404() {
        let body = "Not Found".to_string();
        let res = HttpResponse::new("404", None, Some(body));
        let s: String = res.into();

        assert!(s.starts_with("HTTP/1.1 404 Not Found"));
        assert!(s.contains("Content-Length: 9"));
    }

    // ========== 完整响应格式验证 ==========

    #[test]
    fn test_full_response_format() {
        let mut headers = HashMap::new();
        headers.insert("X-Request-Id", "12345");

        let body = "{\"status\":\"ok\"}".to_string();
        let res = HttpResponse::new("200", Some(headers), Some(body));
        let s: String = res.into();

        // 分行验证
        let lines: Vec<&str> = s.split("\r\n").collect();
        assert_eq!(lines[0], "HTTP/1.1 200 OK");
        assert!(lines[1].contains("X-Request-Id:12345") || lines[2].contains("X-Request-Id:12345"));
        // 找到 Content-Length 行
        let has_content_length = lines.iter().any(|l| l.contains("Content-Length: 15"));
        assert!(has_content_length);

        // 空行分隔
        let empty_line_idx = lines.iter().position(|&l| l.is_empty()).unwrap();
        // body 在空行后
        assert_eq!(lines[empty_line_idx + 1], "{\"status\":\"ok\"}");
    }

    // ========== Clone 测试 ==========

    #[test]
    fn test_clone_response() {
        let body = "test body".to_string();
        let res1 = HttpResponse::new("201", None, Some(body));
        let res2 = res1.clone();

        assert_eq!(res1.status_code(), res2.status_code());
        assert_eq!(res1.body(), res2.body());
    }

    // ========== 边界情况 ==========

    #[test]
    fn test_empty_status_code() {
        // 空字符串状态码应该走 default 分支
        let res = HttpResponse::new("", None, None);
        assert_eq!(res.status_code(), "");
        assert_eq!(res.status_text(), "ERROR");
    }

    #[test]
    fn test_large_body() {
        let large_body = "x".repeat(10000);
        let res = HttpResponse::new("200", None, Some(large_body.clone()));
        let s: String = res.into();

        assert!(s.contains("Content-Length: 10000"));
        assert!(s.ends_with(&large_body));
    }

    #[test]
    fn test_unicode_body() {
        let body = "你好，世界！🦀".to_string();  // 含Rust螃蟹emoji
        let res = HttpResponse::new("200", None, Some(body.clone()));

        // 注意：len() 返回字节数，不是字符数
        let byte_len = body.len();  // UTF-8 编码长度
        let s: String = res.into();

        assert!(s.contains(&format!("Content-Length: {}", byte_len)));
    }
}
