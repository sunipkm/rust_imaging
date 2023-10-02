use warp::{Filter, Rejection, Reply};

// ============================================ PUBLIC =============================================

#[allow(opaque_hidden_inferred_bound)] // https://github.com/rust-lang/rust/issues/107729
pub fn static_files_rules() -> impl Filter<Extract = impl Reply, Error = Rejection> + Copy {
    let index = warp::path::end().map(|| warp::reply::html(INDEX));

    let wasm = warp::path("ccdi-web-client_bg.wasm")
        .map(|| warp::reply::with_header(WASM, "Content-Type", "application/wasm"));

    let js = warp::path("ccdi-web-client.js")
        .map(|| warp::reply::with_header(JS, "Content-Type", "text/javascript"));

    let css = warp::path("index.css")
        .map(|| warp::reply::with_header(CSS, "Content-Type", "text/css"));

    let static_files = wasm.or(js).or(css).or(index);

    static_files
}

// =========================================== PRIVATE =============================================

const INDEX: &[u8] = include_bytes!("static/index.html");
const WASM: &[u8] = include_bytes!("static/ccdi-web-client_bg.wasm");
const JS: &[u8] = include_bytes!("static/ccdi-web-client.js");
const CSS: &[u8] = include_bytes!("static/index.css");
