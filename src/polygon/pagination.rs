use serde::de::DeserializeOwned;

use super::client::PolygonClient;
use super::error::PolygonError;
use super::types::ApiResponse;

pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub next_url: Option<String>,
    pub request_id: Option<String>,
}

impl<T> PagedResponse<T> {
    pub fn has_next(&self) -> bool {
        self.next_url.is_some()
    }
}

pub struct Paginator<'a, T> {
    client: &'a PolygonClient,
    next_url: Option<String>,
    _marker: std::marker::PhantomData<T>,
}

impl<'a, T> Paginator<'a, T>
where
    T: DeserializeOwned + Send,
{
    pub fn new(client: &'a PolygonClient, initial_url: String) -> Self {
        Self {
            client,
            next_url: Some(initial_url),
            _marker: std::marker::PhantomData,
        }
    }

    pub async fn next_page(&mut self) -> Result<Option<PagedResponse<T>>, PolygonError> {
        let url = match self.next_url.take() {
            Some(u) => u,
            None => return Ok(None),
        };

        let response: ApiResponse<ResultsHolder<T>> = self.client.get_raw(&url).await?;

        self.next_url = response.next_url;

        Ok(Some(PagedResponse {
            items: response.data.results,
            next_url: self.next_url.clone(),
            request_id: response.request_id,
        }))
    }

    pub async fn collect_all(mut self) -> Result<Vec<T>, PolygonError> {
        let mut all_items = Vec::new();

        while let Some(page) = self.next_page().await? {
            all_items.extend(page.items);
        }

        Ok(all_items)
    }
}

#[derive(serde::Deserialize)]
struct ResultsHolder<T> {
    #[serde(default = "Vec::new")]
    results: Vec<T>,
}
