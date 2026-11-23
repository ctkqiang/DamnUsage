use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

pub struct SigV4Input<'a> {
    pub method: &'a str,
    pub service: &'a str,
    pub region: &'a str,
    pub host: &'a str,
    pub canonical_uri: &'a str,
    pub canonical_query: &'a str,
    pub extra_headers: &'a [(&'a str, &'a str)],
    pub payload: &'a [u8],
    pub access_key_id: &'a str,
    pub secret_access_key: &'a str,
    pub session_token: Option<&'a str>,
    pub timestamp: DateTime<Utc>,
}

pub struct SignedHeaders {
    pub amz_date: String,
    pub payload_hash: String,
    pub authorization: String,
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("hmac accepts key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

pub fn sign_aws_request(input: &SigV4Input<'_>) -> SignedHeaders {
    let amz_date = input.timestamp.format("%Y%m%dT%H%M%SZ").to_string();
    let date_stamp = input.timestamp.format("%Y%m%d").to_string();
    let payload_hash = sha256_hex(input.payload);

    let mut headers: Vec<(String, String)> = vec![
        ("host".to_string(), input.host.to_string()),
        ("x-amz-content-sha256".to_string(), payload_hash.clone()),
        ("x-amz-date".to_string(), amz_date.clone()),
    ];

    if let Some(token) = input.session_token {
        headers.push(("x-amz-security-token".to_string(), token.to_string()));
    }

    for (k, v) in input.extra_headers {
        headers.push((k.to_lowercase(), v.trim().to_string()));
    }

    headers.sort_by(|a, b| a.0.cmp(&b.0));

    let canonical_headers = headers
        .iter()
        .map(|(k, v)| format!("{k}:{v}\n"))
        .collect::<String>();

    let signed_headers = headers
        .iter()
        .map(|(k, _)| k.as_str())
        .collect::<Vec<_>>()
        .join(";");

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        input.method,
        input.canonical_uri,
        input.canonical_query,
        canonical_headers,
        signed_headers,
        payload_hash
    );

    let credential_scope = format!(
        "{}/{}/{}/aws4_request",
        date_stamp, input.region, input.service
    );

    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        credential_scope,
        sha256_hex(canonical_request.as_bytes())
    );

    let k_secret = format!("AWS4{}", input.secret_access_key);
    let k_date = hmac_sha256(k_secret.as_bytes(), date_stamp.as_bytes());
    let k_region = hmac_sha256(&k_date, input.region.as_bytes());
    let k_service = hmac_sha256(&k_region, input.service.as_bytes());
    let k_signing = hmac_sha256(&k_service, b"aws4_request");
    let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        input.access_key_id, credential_scope, signed_headers, signature
    );

    SignedHeaders {
        amz_date,
        payload_hash,
        authorization,
    }
}
