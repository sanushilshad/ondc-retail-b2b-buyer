// use actix_web::{dev::ServiceRequest, web::BytesMut, HttpMessage};

// fn call(&self, mut req: ServiceRequest) -> Self::Future {
//     let svc = self.service.clone();

//     Box::pin(async move {
//         let mut body = BytesMut::new();
//         let mut stream = req.take_payload();

//         while let Some(chunk) = stream.next().await {
//             body.extend_from_slice(&chunk?);
//         }

//         let obj = serde_json::from_slice::<MyObj>(&body)?;
//         log::info!("{:?}", &obj);

//         //------- Reset the Payload data ----------
//         let (_, mut payload) = Payload::create(true);
//         payload.unread_data(body.into());
//         req.set_payload(payload.into());
//         // ----------------------------------------

//         let res = svc.call(req).await?;

//         Ok(res)
//     })
// }
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::{dev, http::header, Result};
pub fn add_error_header<B>(mut res: dev::ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("Error"),
    );

    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}
