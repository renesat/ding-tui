// use anyhow::Result;
use reqwest::{RequestBuilder, Response, Url};
use serde::de::DeserializeOwned;
use std::future::Future;

use crate::errors::*;
use crate::types::*;

type Result<T, E = DingError> = std::result::Result<T, E>;

pub struct DingClient {
    client: reqwest::Client,
    base_url: Url,
    api_token: String,
}

impl DingClient {
    pub fn new(base_url: Url, api_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_token,
        }
    }

    pub async fn all_bookmarks(&self, params: BookmarksRequest) -> Result<Vec<Bookmark>> {
        self._load_all(params, |p| async { self.bookmarks(p).await })
            .await
    }

    pub async fn bookmarks(&self, params: BookmarksRequest) -> Result<BookmarksResponse> {
        let req = self._bookmarks_request_builder("api/bookmarks/", params)?;
        self._send_request_with_json_output(req).await
    }

    pub async fn all_archived(&self, params: BookmarksRequest) -> Result<Vec<Bookmark>> {
        self._load_all(params, |p| async { self.archived(p).await })
            .await
    }

    pub async fn archived(&self, params: BookmarksRequest) -> Result<BookmarksResponse> {
        let req = self._bookmarks_request_builder("api/bookmarks/archived/", params)?;
        self._send_request_with_json_output(req).await
    }

    pub async fn bookmark(&self, id: u64) -> Result<Bookmark> {
        let req = self._request_builder(reqwest::Method::GET, &format!("api/bookmarks/{id}/"))?;
        self._send_request_with_json_output(req).await
    }

    pub async fn create_bookmark(&self, params: BookmarkRequest) -> Result<Bookmark> {
        assert!(params.url.is_some(), "url need to be specified!");
        let req = self
            ._request_builder(reqwest::Method::POST, "api/bookmarks/")?
            .json(&params);
        self._send_request_with_json_output(req).await
    }

    pub async fn reset_bookmark(&self, id: u64, params: BookmarkRequest) -> Result<Bookmark> {
        assert!(params.url.is_some(), "url need to be specified!");
        let req = self
            ._request_builder(reqwest::Method::PUT, &format!("api/bookmarks/{id}/"))?
            .json(&params);
        self._send_request_with_json_output(req).await
    }

    pub async fn update_bookmark(&self, id: u64, params: BookmarkRequest) -> Result<Bookmark> {
        let req = self
            ._request_builder(reqwest::Method::PATCH, &format!("api/bookmarks/{id}/"))?
            .json(&params);
        self._send_request_with_json_output(req).await
    }

    pub async fn archive_bookmark(&self, id: u64) -> Result<()> {
        let req = self._request_builder(
            reqwest::Method::POST,
            &format!("api/bookmarks/{id}/archive/"),
        )?;
        self._send_request_without_output(req).await
    }

    pub async fn unarchive_bookmark(&self, id: u64) -> Result<()> {
        let req = self._request_builder(
            reqwest::Method::POST,
            &format!("api/bookmarks/{id}/unarchive/"),
        )?;
        self._send_request_without_output(req).await
    }

    pub async fn delete_bookmark(&self, id: u64) -> Result<()> {
        let req =
            self._request_builder(reqwest::Method::DELETE, &format!("api/bookmarks/{id}/"))?;
        self._send_request_without_output(req).await
    }

    pub async fn all_tags(&self, params: TagsRequest) -> Result<Vec<Tag>> {
        self._load_all(params, |p| async { self.tags(p).await })
            .await
    }

    pub async fn tags(&self, params: TagsRequest) -> Result<TagsResponse> {
        let req = self
            ._request_builder(reqwest::Method::GET, "api/tags")?
            .query(&[
                ("limit", params.limit.map(|x| x.to_string())),
                ("offset", params.offset.map(|x| x.to_string())),
            ]);
        self._send_request_with_json_output(req).await
    }

    pub async fn tag(&self, id: u64) -> Result<Tag> {
        let req = self._request_builder(reqwest::Method::GET, &format!("api/tags/{id}/"))?;
        self._send_request_with_json_output(req).await
    }

    pub async fn create_tag(&self, params: TagRequest) -> Result<Tag> {
        let req = self
            ._request_builder(reqwest::Method::POST, "api/tags/")?
            .json(&params);
        self._send_request_with_json_output(req).await
    }

    pub async fn user_profile(&self) -> Result<UserProfile> {
        let req = self._request_builder(reqwest::Method::GET, "api/user/profile/")?;
        self._send_request_with_json_output(req).await
    }

    fn _request_builder(
        &self,
        method: reqwest::Method,
        api_path: &str,
    ) -> Result<reqwest::RequestBuilder> {
        let url = self.base_url.join(api_path)?;
        Ok(self
            .client
            .request(method, url)
            .header("Authorization", format!("Token {0}", self.api_token)))
    }

    fn _bookmarks_request_builder(
        &self,
        api_path: &str,
        params: BookmarksRequest,
    ) -> Result<reqwest::RequestBuilder> {
        Ok(self
            ._request_builder(reqwest::Method::GET, api_path)?
            .query(&[
                ("q", params.query),
                ("limit", params.limit.map(|x| x.to_string())),
                ("offset", params.offset.map(|x| x.to_string())),
            ]))
    }

    async fn _send_request<O: DeserializeOwned, SFut>(
        &self,
        req: RequestBuilder,
        success: impl Fn(Response) -> SFut,
    ) -> Result<O>
    where
        SFut: Future<Output = Result<O>>,
    {
        let resp = req.send().await?.error_for_status()?;
        success(resp).await
    }

    async fn _empty_response_handler(_resp: Response) -> Result<()> {
        Ok(())
    }

    async fn _json_response_handler<O: DeserializeOwned>(resp: Response) -> Result<O> {
        Ok(resp.json().await?)
    }

    async fn _send_request_with_json_output<O: DeserializeOwned>(
        &self,
        req: RequestBuilder,
    ) -> Result<O> {
        self._send_request(req, DingClient::_json_response_handler)
            .await
    }

    async fn _send_request_without_output(&self, req: RequestBuilder) -> Result<()> {
        self._send_request(req, DingClient::_empty_response_handler)
            .await
    }

    async fn _load_all<O, P: IterableRequest, R: IterableResponse<O>, RFut>(
        &self,
        params: P,
        call: impl Fn(P) -> RFut,
    ) -> Result<Vec<O>>
    where
        RFut: Future<Output = Result<R>>,
    {
        let params = params.limit(None).offset(None);
        let mut offset: u64 = 0;
        let mut results = vec![];
        let mut resp = call(params.offset(Some(offset))).await?;
        loop {
            results.extend(resp.results());

            if resp.next().is_none() {
                break;
            }

            offset += resp.results().len() as u64;
            resp = call(params.offset(Some(offset))).await?;
        }
        Ok(results)
    }
}
