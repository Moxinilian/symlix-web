use std::{net::ToSocketAddrs, path::PathBuf, time::Duration};

use anyhow::{anyhow, Context, Result};
use axum::{
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    Router,
};
use notify::{DebouncedEvent, RecursiveMode, Watcher};
use tokio::runtime::Runtime;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use ws::{Message, Sender};

use crate::{generate, Args};

pub struct FileServer {
    hot_reload_sender: Sender,
    _tokio_runtime: Runtime,
}

impl FileServer {
    pub fn new(
        serve_addr: &str,
        serve_folder: PathBuf,
        hot_reload_addr: &str,
        allow_caching: bool,
    ) -> Result<Self> {
        let tokio_runtime = Runtime::new().expect("failed to create tokio runtime");

        let g = tokio_runtime.enter();

        async fn handle_error(_: std::io::Error) -> impl IntoResponse {
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serve files")
        }

        let app = Router::new().fallback(
            axum::routing::get_service(ServeDir::new(serve_folder))
                .handle_error(handle_error)
                .map_response(move |mut res| {
                    if !allow_caching {
                        let headers = res.headers_mut();
                        headers.append(
                            "cache-control",
                            HeaderValue::from_static("no-cache, no-store, must-revalidate"),
                        );
                        headers.append("pragma", HeaderValue::from_static("no-cache"));
                        headers.append("expires", HeaderValue::from_static("0"));
                    }

                    res
                }),
        );

        tokio_runtime.spawn(
            axum::Server::bind(
                &serve_addr
                    .to_socket_addrs()?
                    .next()
                    .ok_or_else(|| anyhow!("invalid server address"))?,
            )
            .serve(app.into_make_service()),
        );

        std::mem::drop(g);

        let hot_reload_server = ws::WebSocket::new(|out: Sender| move |msg| out.send(msg))
            .expect("failed to build websocket");
        let hot_reload_sender = hot_reload_server.broadcaster();

        let hot_reload_socket = hot_reload_addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| anyhow!("invalid hot reloading server address"))?;
        std::thread::spawn(move || {
            hot_reload_server
                .listen(&hot_reload_socket)
                .expect("failed to listen to websocket");
        });

        println!("[INFO] Server opened at http://{}/", serve_addr);

        Ok(Self {
            hot_reload_sender,
            _tokio_runtime: tokio_runtime,
        })
    }

    pub fn reload(&self) {
        self.hot_reload_sender.send(Message::Text("!".into())).ok();
    }
}

impl Drop for FileServer {
    fn drop(&mut self) {
        self.hot_reload_sender.shutdown().ok();
    }
}

pub fn get_dev_html_insert(args: &Args) -> Result<String> {
    let template = include_str!("hot_reload.js");
    let mut tera = tera::Tera::default();
    tera.add_raw_template("hot_reload_snippet", template)
        .expect("bug: failed to load hot reloading snippet template");

    let mut ctx = tera::Context::new();
    ctx.insert("addr", &args.hrs_addr);

    let mut rendered = Vec::with_capacity(template.len());
    tera.render_to("hot_reload_snippet", &ctx, &mut rendered)?;

    let mut out = Vec::new();
    out.extend(b"<script>");
    minify_js::minify(rendered, &mut out).expect("bug: failed to minify hot reloading snippet");
    out.extend(b"</script>");

    Ok(String::from_utf8(out)?)
}

pub fn serve(args: Args) -> Result<()> {
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::watcher(tx, Duration::from_secs(1)).unwrap();

    watcher
        .watch(&args.templates, RecursiveMode::Recursive)
        .unwrap();

    watcher
        .watch(&args.r#static, RecursiveMode::Recursive)
        .unwrap();

    watcher.watch(&args.data, RecursiveMode::Recursive).unwrap();

    let remote = FileServer::new(
        &args.addr,
        args.output.clone(),
        &args.hrs_addr,
        args.allow_caching,
    )
    .context("failed to start development server")?;

    loop {
        match rx.recv() {
            Ok(
                DebouncedEvent::Write(_)
                | DebouncedEvent::Create(_)
                | DebouncedEvent::Rename(_, _)
                | DebouncedEvent::Remove(_),
            ) => {
                if let Err(e) = generate(&args) {
                    eprintln!("\x1b[31m[ERROR] {:#}\x1b[0m", e);
                } else {
                    remote.reload();
                }
            }
            Ok(DebouncedEvent::NoticeWrite(_) | DebouncedEvent::NoticeRemove(_)) => {
                println!("[INFO] Change detected. Preparing rebuild.");
            }
            Ok(_) => (),
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }
}
