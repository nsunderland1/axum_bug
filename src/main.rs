fn main() {}

#[cfg(test)]
mod tests {
    use std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    };

    use async_trait::async_trait;
    use axum::{
        body::Body, error_handling::HandleErrorLayer, extract::Extension, http::Method,
        routing::get, Router,
    };
    use http::StatusCode;
    use tokio::sync::Barrier;
    use tower::{load_shed::error::Overloaded, Service, ServiceBuilder, ServiceExt};

    async fn error_mapper(err: Box<dyn std::error::Error + Send + Sync>) -> StatusCode {
        if err.is::<Overloaded>() {
            StatusCode::TOO_MANY_REQUESTS
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }

    /// An internal trait. We create a single instance of this, `Arc` it, and share it as an Extension across all routes
    #[async_trait]
    trait Backend: Send + Sync {
        async fn foo(&self);
    }

    fn app(backend: Arc<dyn Backend>, request_max_concurrency: usize) -> Router {
        Router::new()
            .route("/foo", get(handle_foo))
            .layer(Extension(backend))
            .layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(error_mapper))
                    .load_shed()
                    // We want to test this concurrency limit layer below
                    .concurrency_limit(request_max_concurrency),
            )
    }

    async fn handle_foo(backend: Extension<Arc<dyn Backend>>) {
        backend.foo().await
    }

    // A test for our concurrency limiting
    #[tokio::test]
    async fn admission_control() {
        const CONCURRENCY: usize = 2;

        // This mock backend will be used to verify the concurrency limit
        #[derive(Default)]
        struct MockBackend {
            counter: AtomicUsize,
        }

        #[async_trait]
        impl Backend for MockBackend {
            async fn foo(&self) {
                // This function gets called on each request, so if we ever trigger this assertion,
                // that means our concurrency limit layer isn't doing its job.
                let cur = self.counter.fetch_add(1, Ordering::SeqCst);
                assert!(cur < CONCURRENCY);
                // Wait a really long time to ensure that we don't finish the request before other requests start
                tokio::time::sleep(Duration::from_secs(5)).await;
                self.counter.fetch_sub(1, Ordering::SeqCst);
            }
        }

        let backend = MockBackend::default();

        let app = app(Arc::new(backend), CONCURRENCY);
        let barrier = Arc::new(Barrier::new(10));

        // Spawn 10 concurrent tasks, each of which will send a `foo` request
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let barrier = barrier.clone();
                let mut app = app.clone();
                tokio::task::spawn(async move {
                    barrier.wait().await;

                    let request = http::Request::builder()
                        .method(Method::GET)
                        .uri("/foo")
                        .body(Body::empty())
                        .unwrap();

                    let response = app.ready().await.unwrap().call(request).await.unwrap();

                    response.status()
                })
            })
            .collect();

        // If the concurrency limit works properly, we only expect 2 of these to succeed.
        // The rest should get locked out by the concurrency limit and return TOO_MANY_REQUESTS
        let mut responses = Vec::new();
        for handle in handles {
            responses.push(handle.await.unwrap());
        }
        let (ok, not_ok): (Vec<_>, Vec<_>) = responses
            .into_iter()
            .partition(|&status| status == StatusCode::OK);
        assert_eq!(ok.len(), CONCURRENCY);
        assert!(not_ok
            .iter()
            .all(|&status| status == StatusCode::TOO_MANY_REQUESTS));
    }
}
