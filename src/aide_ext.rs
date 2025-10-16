use aide::axum::ApiRouter;
pub trait ApiRouterExt {
    fn into_api_router(self) -> ApiRouter;
    fn api_tag(self, tag: &'static str) -> ApiRouter
    where
        Self: Sized,
    {
        self.into_api_router().with_path_items(|t| t.tag(tag))
    }
}
impl ApiRouterExt for ApiRouter {
    fn into_api_router(self) -> ApiRouter {
        self
    }
}
