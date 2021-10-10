use warp::{http::StatusCode, sse::Event, Filter};
use tokio::sync::{oneshot, mpsc};

pub async fn run_editor(shutdown: oneshot::Receiver<i32>) {
    let index = warp::path::end().map(|| {
        return "index";
    });

    let app = index;

    log::info!("www running");
    let (_, run) = warp::serve(app).bind_with_graceful_shutdown(([0, 0, 0, 0], 3732), async move {
        shutdown.await.ok();
    });
    run.await;
    log::info!("www stopped");
}
