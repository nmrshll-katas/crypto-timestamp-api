use crate::routes::middleware::pow_ratelimit::solve_pow_proof_b64;

// Happy path
#[tokio::test]
async fn test__sign_data__OK() -> Result<(), anyhow::Error> {
    let data_bytes = b"hello dog this is data";
    let res = warp::test::request()
        .method("POST")
        .path("/sign_data")
        .body(format!(
            r#"{{"data_base64":"{}","pow_proof_base64":"{}"}}"#,
            base64::encode(&data_bytes),
            solve_pow_proof_b64(data_bytes)
        ))
        .reply(&crate::router()) // Server routes to respond with
        .await;
    let sd_resp: crate::routes::SignDataResp = serde_json::from_slice(&res.body())?;

    assert_eq!(res.status(), 200, "Should return 200 OK");

    // verify the signature: we'll need {data_base64, timestamp} in json bytes, then hashed
    let fields_signed = sd_resp.fields_signed;
    let json_bytes: Vec<u8> = serde_json::to_vec(&fields_signed)?;
    let fields_signed_hash = blake3::hash(&json_bytes);

    // with fields_signed_hash being the message, the server's pubkey and the signature, we can verify:
    let signature_bytes = base64::decode(&sd_resp.signature_base64)?;
    let sig_ok = crate::config::keypair().verify(fields_signed_hash.as_bytes(), &signature_bytes);

    assert_eq!(sig_ok, true, "failed verifying signature");
    Ok(())
}

// GET: Method not allowed
#[tokio::test]
async fn test__sign_data__WrongMethod() -> Result<(), anyhow::Error> {
    let res = warp::test::request()
        .method("GET")
        .path("/sign_data")
        .reply(&crate::router()) // Server routes to respond with
        .await;

    assert_eq!(res.status(), 405, "Should return 405 Method not Allowed");
    assert_eq!(
        res.body(),
        r#"{"code":405,"message":"Method not Allowed","status":"error"}"#
    );
    Ok(())
}

// Invalid body: missing field data_base64
#[tokio::test]
async fn test__sign_data__InvalidBody_missingField() -> Result<(), anyhow::Error> {
    let res = warp::test::request()
        .method("POST")
        .path("/sign_data")
        .body(r#"{"hello":"world"}"#)
        .reply(&crate::router()) // Server routes to respond with
        .await;

    assert_eq!(res.status(), 400, "Should return 400 Bad Request");
    assert_eq!(
        res.body(),
        r#"{"code":400,"message":"Bad Request: Request body deserialize error: missing field `data_base64` at line 1 column 17","status":"error"}"#
    );
    Ok(())
}

// Invalid body: missing field pow_proof_base64
#[tokio::test]
async fn test__sign_data__InvalidBody_missingField2() -> Result<(), anyhow::Error> {
    let data_b64 = base64::encode(b"hello dog this is different data");
    let res = warp::test::request()
        .method("POST")
        .path("/sign_data")
        .body(format!(r#"{{"data_base64":"{}"}}"#, data_b64))
        .reply(&crate::router()) // Server routes to respond with
        .await;

    assert_eq!(res.status(), 400, "Should return 400 Bad Request");
    // assert_eq!(
    //     res.body(),
    //     r#"{"code":400,"message":"Bad Request: Request body deserialize error: missing field `pow_proof_base64` at line 1 column 17","status":"error"}"#
    // );
    Ok(())
}

// Invalid base64 in body.data_base64
#[tokio::test]
async fn test__sign_data__InvalidBody_invalidBase64() -> Result<(), anyhow::Error> {
    let data_str = "hello world"; // Invalid base64: contains a space
    let res = warp::test::request()
        .method("POST")
        .path("/sign_data")
        .body(format!(
            r#"{{"data_base64":"{}","pow_proof_base64":"{}"}}"#,
            &data_str, // passed directly as string, should be passed as base64 instead => server returns 400
            solve_pow_proof_b64(data_str.as_bytes())
        ))
        .reply(&crate::router()) // Server routes to respond with
        .await;

    assert_eq!(res.status(), 400, "Should return 400 Bad Request");
    assert_eq!(
        res.body(),
        r#"{"code":400,"message":"Invalid base64 field: Invalid byte 32, offset 5.","status":"error"}"#
    );
    Ok(())
}

// signed_data already exists in DB
#[tokio::test]
async fn test__sign_data__AlreadyExists() -> Result<(), anyhow::Error> {
    let data_bytes = b"test__sign_data__AlreadyExists";

    let _res = warp::test::request()
        .method("POST")
        .path("/sign_data")
        .body(format!(
            r#"{{"data_base64":"{}","pow_proof_base64":"{}"}}"#,
            base64::encode(&data_bytes),
            solve_pow_proof_b64(data_bytes)
        ))
        .reply(&crate::router()) // Server routes to respond with
        .await;
    let res = warp::test::request()
        .method("POST")
        .path("/sign_data")
        .body(format!(
            r#"{{"data_base64":"{}","pow_proof_base64":"{}"}}"#,
            base64::encode(&data_bytes),
            solve_pow_proof_b64(data_bytes)
        ))
        .reply(&crate::router()) // Server routes to respond with
        .await;

    assert_eq!(res.status(), 409, "Should return 409 Conflict");
    assert_eq!(
        res.body(),
        r#"{"code":409,"message":"Resource already exists","status":"error"}"#
    );
    Ok(())
}

// Bad Request: pow_proof rejected
#[tokio::test]
async fn test__sign_data__PowProof_rejected() -> Result<(), anyhow::Error> {
    let data_bytes = b"6dfgs7896d7fgiuyfkgfsdyiguhk";
    let no_proof_base64 = base64::encode(b"this-is-not-a-proof");
    let res = warp::test::request()
        .method("POST")
        .path("/sign_data")
        .body(format!(
            r#"{{"data_base64":"{}","pow_proof_base64":"{}"}}"#,
            base64::encode(&data_bytes),
            no_proof_base64
        ))
        .reply(&crate::router()) // Server routes to respond with
        .await;

    assert_eq!(res.status(), 400, "Should return 400 Bad Request");
    assert_eq!(
        res.body(),
        r#"{"code":400,"message":"PoW proof didn't pass verification","status":"error"}"#
    );

    Ok(())
}
