use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct GenericResponse {
    pub status: bool,
    pub customer_message: String,
    pub code: String,
}

// impl Responder for GenericResponse {
//     fn respond_to(self, _req: &web::HttpRequest) -> HttpResponse {
//         HttpResponse::Ok().json(self)
//     }
// }

impl GenericResponse {
    // Associated function for creating a success response
    pub fn success(message: &str) -> Self {
        Self {
            status: true,
            customer_message: String::from(message),
            code: String::from("200"),
        }
    }

    // Associated function for creating an error response
    pub fn error(message: &str, code: &str) -> Self {
        Self {
            status: false,
            customer_message: String::from(message),
            code: String::from(code),
        }
    }
}
