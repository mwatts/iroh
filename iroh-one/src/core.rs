use crate::uds;
use axum::{Router, Server};
use iroh_gateway::{core::State, handlers::get_app_routes};
use std::sync::Arc;
use tokio::net::UnixListener;

pub fn uds_server(
    state: Arc<State>,
) -> Server<
    uds::ServerAccept,
    axum::extract::connect_info::IntoMakeServiceWithConnectInfo<Router, uds::UdsConnectInfo>,
> {
    #[cfg(target_os = "android")]
    let path = "/dev/socket/ipfsd.http".to_owned();

    #[cfg(not(target_os = "android"))]
    let path = format!("{}", std::env::temp_dir().join("ipfsd.http").display());

    let _ = std::fs::remove_file(&path);
    let uds = UnixListener::bind(&path).unwrap();
    println!("Binding to UDS at {}", path);
    let app = get_app_routes(&state);
    Server::builder(uds::ServerAccept { uds })
        .serve(app.into_make_service_with_connect_info::<uds::UdsConnectInfo>())
}
