pub mod sse_loader {
    use serde::Deserialize;
    use serde_json::from_str;

    use futures::stream::StreamExt;
    use futures_core::Stream;

    pub async fn load_stream<T>(
        url: &str,
    ) -> Result<impl Stream<Item = T>, reqwest::Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        let res = reqwest::get(url)
            .await?
            .bytes_stream()
            .filter_map(|it| async {
                it.ok()
                    .map(|bytes| {
                        std::str::from_utf8(&bytes)
                            .ok()
                            .map(|result_str| {
                                let result_string = result_str.to_string();
                                if result_string.starts_with("data") {
                                    let sub_str = &result_string[5..];
                                    Some(sub_str.to_string())
                                } else if result_string.starts_with("{") {
                                    Some(result_string)
                                } else {
                                    None
                                }
                            })
                            .flatten()
                    })
                    .flatten()
            })
            .filter_map(|it: String| async move { from_str::<T>(&it).ok() });
        Ok(Box::pin(res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::StreamExt;
    use serde::Deserialize;
    use tokio;
    #[derive(Deserialize)]
    struct StockPrice {}

    #[tokio::test]
    async fn test_stream() {
        let stream = sse_loader::load_stream::<StockPrice>(
            "https://api.boerse-frankfurt.de/data/price_information?isin=IE00B0M62Q58&mic=XETR",
        )
        .await;
        assert!(stream.is_ok());
        let val = stream.unwrap().next().await;
        assert!(val.is_some());
    }
}
