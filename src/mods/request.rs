pub async fn async_get_bytes(url: &str) -> Result<Vec<u8>, ()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header(
            "User-Agent",
            format!("bangumi-rss-proxy/{}", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await
        .map_err(|_| ())?;
    let mut bytes: Vec<u8> = vec![];
    let resp = resp.bytes().await.map_err(|_| ())?;
    resp.into_iter().for_each(|b| bytes.push(b));
    Ok(bytes)
}

pub async fn async_get_string(url: &str) -> Result<String, ()> {
    let client = reqwest::Client::new();
    let resp = client
        .get(url)
        .header(
            "User-Agent",
            format!("bangumi-rss-proxy/{}", env!("CARGO_PKG_VERSION")),
        )
        .send()
        .await
        .map_err(|_| ())?;
    let resp_text = resp.text().await.map_err(|_| ())?;
    Ok(resp_text)
}
