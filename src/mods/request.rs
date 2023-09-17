pub async fn async_get_bytes(url: &str) -> Result<Vec<u8>, ()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header("User-Agent", "bangumi-rss-proxy")
        .send()
        .await
        .map_err(|_| ())?;
    let mut bytes: Vec<u8> = vec![];
    let resp = resp.bytes().await.map_err(|_| ())?;
    resp.into_iter().for_each(|b| bytes.push(b));
    Ok(bytes)
}
