use crate::state::State;
use async_std::sync::Arc;
use tide::utils::async_trait;
use tide::{Middleware, Next, Request, Response, Result, StatusCode};

pub struct AuthMiddleware;

#[async_trait]
impl Middleware<Arc<State>> for AuthMiddleware {
    async fn handle(&self, req: Request<Arc<State>>, next: Next<'_, Arc<State>>) -> Result {
        if !req
            .header("x-api-key")
            .map(|h| h.as_str() == req.state().settings.api_key)
            .unwrap_or(false)
        {
            let mut res = Response::new(StatusCode::Unauthorized);
            res.set_body("invalid api key");
            return Ok(res);
        }

        Ok(next.run(req).await)
    }
}
