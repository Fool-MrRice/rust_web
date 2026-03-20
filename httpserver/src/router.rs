use super::handler::{Handler, PageNotFoundHandler, StaticPageHandler, WebServiceHandler};
use http::httprequest::{HttpMethod, HttpReSource, HttpRequest};
use http::httpresponse::HttpResponse;
use std::io::Write;

/// Router结构体/对象，仅负责将传入的HttpRequest进行分流给不同handler处理（需要同时传入同一个TcpStream保持通信）
pub struct Router;
impl Router {
    // route不返回任何值，仅仅通过&mut 进行写入和返回
    pub fn route(http_request: HttpRequest, stream: &mut impl Write) {
        // 第一步解析request方法：Get和_
        // 第二步解析request请求资源（需要进一步解析请求不同资源）
        // 最后都是要进行handler处理得到返回的HttpResponse，并通过传入的stream返回
        match http_request.method() {
            HttpMethod::Get => {
                match &http_request.resource() {
                    HttpReSource::Path(s) => {
                        let route: Vec<&str> = s.split("/").collect();
                        match route[1] {
                            "api" => {
                                let response: HttpResponse =
                                    WebServiceHandler::handle(&http_request);
                                let _ = response.send_response(stream);
                            }
                            _ => {
                                let response: HttpResponse =
                                    StaticPageHandler::handle(&http_request);
                                let _ = response.send_response(stream);
                            }
                        }
                    } // _ => {
                      //     let response: HttpResponse = PageNotFoundHandler::handle(&http_request);
                      //     let _ = response.send_response(stream);
                      // }
                }
            }
            _ => {
                let response: HttpResponse = PageNotFoundHandler::handle(&http_request);
                let _ = response.send_response(stream);
            } // HttpMethod::Post => {}
              // HttpMethod::Uninitialized=>{}
        }
    }
}
