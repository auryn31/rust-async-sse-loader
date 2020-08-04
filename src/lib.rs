/// Load and parse data from an url that serves them via sse
///
/// # Examples
///
/// ```
/// let url = "https://example.com";
/// let answer = sse_loader::load_stream(arg);
///
/// ```
pub mod sse_loader {
    use serde::{Deserialize};
    use serde_json::from_str;

    use futures::future;
    use futures::stream::StreamExt;
    use futures_core::Stream;

    pub async fn load_stream<T>(url: &str) -> Result<impl Stream<Item = T>, reqwest::Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        let res = reqwest::get(url)
            .await?
            .bytes_stream()
            .filter(|it| future::ready(it.is_ok()))
            .map(|it| {
                let bytes = it.unwrap();
                if let Ok(result_str) = std::str::from_utf8(&bytes) {
                    let result_string = result_str.to_string();
                    if result_string.starts_with("data") {
                        let sub_str = &result_string[5..];
                        sub_str
                    } else if result_string.starts_with("{") {
                        &result_string
                    } else {
                        ""
                    }.to_string()
                } else {
                    "".to_string()
                }
            })
            .filter(|it| future::ready(it.len() > 1))
            .map(|it| from_str::<T>(&it))
            .filter(|it| future::ready(it.is_ok()))
            .map(|it| it.unwrap());
        Ok(res)
    }
}