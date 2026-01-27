use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{Value, json};

use crate::mcp::types::{CallToolResult, Tool, ToolCallFuture, ToolDefinition, ToolInputSchema};
use crate::polygon::{PolygonClient, PolygonError};

use super::types::NewsResponse;

pub struct GetNews {
    client: Arc<PolygonClient>,
}

impl GetNews {
    pub fn new(client: Arc<PolygonClient>) -> Self {
        Self { client }
    }

    async fn execute(&self, args: HashMap<String, Value>) -> Result<CallToolResult, PolygonError> {
        let mut params = Vec::new();

        if let Some(ticker) = args.get("ticker").and_then(|v| v.as_str()) {
            params.push(format!("ticker={}", ticker.to_uppercase()));
        }

        if let Some(published_utc) = args.get("published_utc").and_then(|v| v.as_str()) {
            params.push(format!("published_utc={}", published_utc));
        }

        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10);
        params.push(format!("limit={}", limit));

        if let Some(order) = args.get("order").and_then(|v| v.as_str()) {
            params.push(format!("order={}", order));
        }

        if let Some(sort) = args.get("sort").and_then(|v| v.as_str()) {
            params.push(format!("sort={}", sort));
        }

        let url = format!("/v2/reference/news?{}", params.join("&"));

        let response: NewsResponse = self.client.get(&url).await?;

        let articles: Vec<Value> = response
            .results
            .iter()
            .map(|a| {
                json!({
                    "id": a.id,
                    "title": a.title,
                    "author": a.author,
                    "publishedUtc": a.published_utc,
                    "articleUrl": a.article_url,
                    "imageUrl": a.image_url,
                    "description": a.description,
                    "tickers": a.tickers,
                    "keywords": a.keywords,
                    "publisher": a.publisher.as_ref().map(|p| json!({
                        "name": p.name,
                        "homepageUrl": p.homepage_url,
                        "logoUrl": p.logo_url
                    }))
                })
            })
            .collect();

        let result = json!({
            "count": response.count,
            "articles": articles
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?).with_structured(result))
    }
}

impl Tool for GetNews {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new("get_news")
            .with_description(
                "Get news articles for stocks. Returns title, author, published date, \
                article URL, related tickers, and description.",
            )
            .with_schema(ToolInputSchema {
                schema_type: "object".to_string(),
                properties: Some(HashMap::from([
                    (
                        "ticker".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by ticker symbol (e.g., AAPL, TSLA)"
                        }),
                    ),
                    (
                        "published_utc".to_string(),
                        json!({
                            "type": "string",
                            "description": "Filter by publish date (YYYY-MM-DD or ISO datetime)"
                        }),
                    ),
                    (
                        "limit".to_string(),
                        json!({
                            "type": "integer",
                            "description": "Max articles to return (default 10)",
                            "default": 10
                        }),
                    ),
                    (
                        "order".to_string(),
                        json!({
                            "type": "string",
                            "description": "Sort order: asc or desc",
                            "enum": ["asc", "desc"]
                        }),
                    ),
                    (
                        "sort".to_string(),
                        json!({
                            "type": "string",
                            "description": "Field to sort by: published_utc",
                            "default": "published_utc"
                        }),
                    ),
                ])),
                required: None,
            })
    }

    fn call(&self, arguments: HashMap<String, Value>) -> ToolCallFuture {
        let this = Self {
            client: self.client.clone(),
        };
        Box::pin(async move {
            this.execute(arguments)
                .await
                .map_err(|e| crate::mcp::McpError::InternalError {
                    message: e.to_string(),
                })
        })
    }
}
