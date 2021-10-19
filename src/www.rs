use warp::{http::StatusCode, sse::Event, Filter};
use tokio::sync::{oneshot, mpsc};
use tera::{Context as TeraContext, Tera};
use crate::exam;

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

macro_rules! str_decode {
    (@inner $ident:ident, $e:ident, $return:expr) => {
        let $ident = match urlencoding::decode(&$ident) {
            Ok(v) => v,
            Err($e) => {
                log::error!("urldecode {} error: {}", &$ident, &$e);
                return $return;
            }
        };
    };

    ($ident:ident) => {
        str_decode!(@inner $ident, e, render!(@errhtml "URLDecode", &e.to_string()))
    };
    (@jsn $ident:ident) => {
        str_decode!(@inner $ident, e, warp::reply::json(&format!("URLDecode错误：{}", &e)))
    };
}

pub async fn run_editor(shutdown: oneshot::Receiver<i32>) {
    let index = warp::path::end().map(|| {
        render!("editor_index.html", &TeraContext::new())
    });

    let editor = warp::path("editor");
    let editor_clustering = warp::path!("1" / String)
        .map(|name: String| {
            str_decode!(name);
            let mut ctx = TeraContext::new();
            let data: exam::ClusteringExam = super::load_test_data("1", &name)
                .unwrap_or_else(|e| {
                    log::error!("load ClusteringExam of {} error: {}", name, e);
                    Default::default()
                });
            let col = data.column_count();
            let tbl = data.table();
            ctx.insert("data", &tbl);
            ctx.insert("column", &col);
            render!("editor_clustering.html", &ctx)
        });

    let card = warp::path("card");
    let card_testnameopts = warp::path!("test_name_option" / String)
        .map(|typ| {
            let mut ctx = TeraContext::new();
            let names = super::list_names_of_test_type(typ);
            ctx.insert("opts", &names);
            render!("test_name_option.html", &ctx)
        });

    let submit = warp::path("submit");
    let submit_clustering = warp::path!("1" / String)
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|name: String, opt: exam::ClusteringExam| {
            str_decode!(@jsn name);
            let desc = match super::commit_test_data("1", &name, &opt) {
                Ok(_) => {
                    log::info!("save test type clustering {}", &name);
                    String::from("保存成功")
                }
                Err(e) => {
                    log::error!("save test type clustering {} error: {}", &name, &e);
                    format!("保存出错：{}", e)
                }
            };
            warp::reply::json(&desc)
        });

    let static_files = warp::path("static").and(warp::fs::dir("./static"));
    let favicon = warp::path!("favicon.ico").and(warp::fs::file("./static/favicon.ico"));

    let app = index
        .or(editor.and(editor_clustering))
        .or(card.and(card_testnameopts))
        .or(submit.and(submit_clustering))
        .or(static_files)
        .or(favicon);

    log::info!("www running");
    let (_, run) = warp::serve(app).bind_with_graceful_shutdown(([0, 0, 0, 0], 3732), async move {
        shutdown.await.ok();
    });
    run.await;
    log::info!("www stopped");
}
