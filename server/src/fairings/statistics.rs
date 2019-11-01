// use std::io::Cursor;
// use std::sync::atomic::{AtomicUsize, Ordering};

// use rocket::fairing::{Fairing, Info, Kind};
// use rocket::http::{ContentType, Method, Status};
// use rocket::{Data, Request, Response};

// /// Implements basic statistics
// pub struct Statistics {
//     get: AtomicUsize,
//     post: AtomicUsize,
// }

// impl Default for Statistics {
//     fn default() -> Statistics {
//         Statistics {
//             get: AtomicUsize::new(0),
//             post: AtomicUsize::new(0),
//         }
//     }
// }

// impl Fairing for Statistics {
//     fn info(&self) -> Info {
//         Info {
//             name: "GET/POST Counter",
//             kind: Kind::Request | Kind::Response,
//         }
//     }

//     fn on_request(&self, request: &mut Request, _: &Data) {
//         if request.method() == Method::Get {
//             self.get.fetch_add(1, Ordering::Relaxed);
//         } else if request.method() == Method::Post {
//             self.post.fetch_add(1, Ordering::Relaxed);
//         }
//     }

//     fn on_response(&self, request: &Request, response: &mut Response) {
//         // Don't change a successful user's response, ever.
//         if response.status() != Status::NotFound {
//             return;
//         }
//         // TODO: Expose dedicated /stats route via request-local cache
//         if request.method() == Method::Get && request.uri().path() == "/stats" {
//             let get_count = self.get.load(Ordering::Relaxed);
//             let post_count = self.post.load(Ordering::Relaxed);

//             let body = format!("Get: {}\nPost: {}", get_count, post_count);
//             response.set_status(Status::Ok);
//             response.set_header(ContentType::Plain);
//             response.set_sized_body(Cursor::new(body));
//         }
//     }
// }
