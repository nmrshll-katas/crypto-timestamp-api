mod pubkey;
mod sign_data;

#[tokio::test]
async fn test_getRoot() {
    let res = warp::test::request()
        .method("GET")
        .path("/")
        .reply(&crate::router()) // Server routes to respond with
        .await;

    assert_eq!(res.status(), 200, "Should return 200 OK.");
    assert_eq!(res.body(), "Hello world !");
}

#[tokio::test]
async fn test_health() {
    let res = warp::test::request()
        .method("GET")
        .path("/health")
        .reply(&crate::router())
        .await;

    assert_eq!(res.status(), 200, "Should return 200 OK.");
    assert_eq!(res.body(), "Hello world !");
}
