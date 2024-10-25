use crate::helpers::spawn_app;

#[actix_web::test]
async fn health_check_works() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();
    // Act
    let response = client
        .get(&format!("{}/util/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(14), response.content_length());
}
