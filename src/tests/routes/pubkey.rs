// Happy path
#[tokio::test]
async fn test__pubkey__OK() -> Result<(), anyhow::Error> {
    let res = warp::test::request()
        .method("GET")
        .path("/pubkey")
        .reply(&crate::router()) // Server routes to respond with
        .await;
    let pk_resp: crate::routes::PubkeyResp = serde_json::from_slice(&res.body())?;

    assert_eq!(res.status(), 200, "Should return 200 OK.");
    assert_eq!(
        pk_resp.pubkey.to_bytes(),
        crate::config::keypair().pubkey().to_bytes(),
        "pubkey should be same as in config"
    );
    assert_eq!(
        pk_resp.pubkey.to_bytes().len(),
        32,
        "pubkey should be 32 bytes long"
    );
    Ok(())
}

// POST: Method not allowed
#[tokio::test]
async fn test__pubkey__WrongMethod() -> Result<(), anyhow::Error> {
    let res = warp::test::request()
        .method("POST")
        .path("/pubkey")
        .reply(&crate::router()) // Server routes to respond with
        .await;

    assert_eq!(res.status(), 405, "Should return 405 Method not Allowed.");
    assert_eq!(
        res.body(),
        r#"{"code":405,"message":"Method not Allowed","status":"error"}"#
    );
    Ok(())
}
