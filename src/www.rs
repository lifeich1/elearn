use warp::{http::StatusCode, sse::Event, Filter};
use tokio::sync::{oneshot, mpsc};
use tera::{Context as TeraContext, Tera};

lazy_static::lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let tera = match Tera::new("templates/**/*.html") {
            Ok(t) => t,
            Err(e) => {
                log::error!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera
    };
}

macro_rules! render {
    (@errhtml $kind:expr, $reason:expr) => {
        warp::reply::html(render!(@err TEMPLATES, $kind, $reason))
    };
    (@err $tera:ident, $kind:expr, $reason:expr) => {
        {
            let mut ctx = TeraContext::new();
            ctx.insert("kind", $kind);
            ctx.insert("reason", $reason);
            log::error!("render error: kind {}, reason {}", $kind, $reason);
            $tera.render("failure.html", &ctx).unwrap()
        }
    };

    ($name:expr, $ctx:expr) => {
        render!(TEMPLATES, $name, $ctx)
    };
    ($tera:ident, $name:expr, $ctx:expr) => {
        warp::reply::html($tera.render($name, $ctx).unwrap_or_else(|e|
            render!(@err $tera, "Tera engine", &format!("Error: tera: {}", e))
        ))
    };
}

pub async fn run_editor(shutdown: oneshot::Receiver<i32>) {
    let index = warp::path::end().map(|| {
        render!("editor_index.html", &TeraContext::new())
    });

    let app = index;

    log::info!("www running");
    let (_, run) = warp::serve(app).bind_with_graceful_shutdown(([0, 0, 0, 0], 3732), async move {
        shutdown.await.ok();
    });
    run.await;
    log::info!("www stopped");
}
