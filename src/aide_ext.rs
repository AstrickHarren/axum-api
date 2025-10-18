use aide::axum::ApiRouter;
pub trait ApiRouterExt<S: Clone + Sync + Send> {
    fn into_api_router(self) -> ApiRouter<S>;
    fn api_tag(self, tag: &'static str) -> ApiRouter<S>
    where
        Self: Sized,
        S: 'static,
    {
        self.into_api_router().with_path_items(|t| t.tag(tag))
    }
}
impl<S: Clone + Sync + Send> ApiRouterExt<S> for ApiRouter<S> {
    fn into_api_router(self) -> ApiRouter<S> {
        self
    }
}
