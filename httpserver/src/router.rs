use super::handler::{Handler, PageNotFoundHandler, StaticPageHandler, WebServiceHandler};
use http::httprequest::{HttpMethod, HttpReSource, HttpRequest};
use http::httpresponse::HttpResponse;
use std::io::Write;

pub struct Router;
impl Router {
    pub fn route(http_request: HttpRequest, stream: &mut impl Write) {
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
