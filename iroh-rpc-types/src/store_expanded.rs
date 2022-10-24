pub use super::store_proto::*;
pub async fn serve<T: Store>(addr: StoreServerAddr, source: T) -> anyhow::Result<()> {
    match addr {
        #[cfg(feature = "grpc")]
        crate::Addr::GrpcHttp2(addr) => {
            let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
            health_reporter
                .set_serving::<store_server::StoreServer<T>>()
                .await;
            tonic::transport::Server::builder()
                .add_service(health_service)
                .add_service(store_server::StoreServer::new(source))
                .serve(addr)
                .await?;
            Ok(())
        }
        #[cfg(all(feature = "grpc", unix))]
        crate::Addr::GrpcUds(path) => {
            use anyhow::Context;
            use tokio::net::UnixListener;
            use tokio_stream::wrappers::UnixListenerStream;
            let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
            health_reporter
                .set_serving::<store_server::StoreServer<T>>()
                .await;
            if path.exists() {
                if path.is_dir() {
                    return ::anyhow::__private::Err(::anyhow::Error::msg({
                        let res = format!("cannot bind socket to directory: {:?}", path,);
                        res
                    }));
                } else {
                    return ::anyhow::__private::Err(::anyhow::Error::msg({
                        let res = format!("cannot bind socket: already exists: {:?}", path);
                        res
                    }));
                }
            }
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    return ::anyhow::__private::Err(::anyhow::Error::msg({
                        let res = format!("socket parent directory doesn\'t exist: {:?}", parent,);
                        res
                    }));
                }
            }
            struct UdsGuard(std::path::PathBuf);
            impl Drop for UdsGuard {
                fn drop(&mut self) {
                    let _ = std::fs::remove_file(&self.0);
                }
            }
            let uds = UnixListener::bind(&path).with_context(|| {
                let res = format!("failed to bind to {:?}", path);
                res
            })?;
            let _guard = UdsGuard(path.clone().into());
            let uds_stream = UnixListenerStream::new(uds);
            tonic::transport::Server::builder()
                .add_service(health_service)
                .add_service(store_server::StoreServer::new(source))
                .serve_with_incoming(uds_stream)
                .await?;
            Ok(())
        }
        #[cfg(feature = "mem")]
        crate::Addr::Mem(mut receiver) => {
            while let Some((msg, sender)) = receiver.recv().await {
                match msg {
                    StoreRequest::version(req) => {
                        let res = source.version(req).await.map_err(|e| e.to_string());
                        sender.send(StoreResponse::version(res)).ok();
                    }
                    StoreRequest::put(req) => {
                        let res = source.put(req).await.map_err(|e| e.to_string());
                        sender.send(StoreResponse::put(res)).ok();
                    }
                    StoreRequest::get(req) => {
                        let res = source.get(req).await.map_err(|e| e.to_string());
                        sender.send(StoreResponse::get(res)).ok();
                    }
                    StoreRequest::has(req) => {
                        let res = source.has(req).await.map_err(|e| e.to_string());
                        sender.send(StoreResponse::has(res)).ok();
                    }
                    StoreRequest::get_links(req) => {
                        let res = source.get_links(req).await.map_err(|e| e.to_string());
                        sender.send(StoreResponse::get_links(res)).ok();
                    }
                    StoreRequest::get_size(req) => {
                        let res = source.get_size(req).await.map_err(|e| e.to_string());
                        sender.send(StoreResponse::get_size(res)).ok();
                    }
                }
            }
            Ok(())
        }
    }
}
pub type StoreServerAddr = crate::Addr<
    tokio::sync::mpsc::Receiver<(StoreRequest, tokio::sync::oneshot::Sender<StoreResponse>)>,
>;
pub type StoreClientAddr = crate::Addr<
    tokio::sync::mpsc::Sender<(StoreRequest, tokio::sync::oneshot::Sender<StoreResponse>)>,
>;
#[allow(clippy::large_enum_variant)]
pub enum StoreClientBackend {
    #[cfg(feature = "grpc")]
    Grpc {
        client: store_client::StoreClient<tonic::transport::Channel>,
        health: tonic_health::proto::health_client::HealthClient<tonic::transport::Channel>,
    },
    #[cfg(feature = "mem")]
    Mem(tokio::sync::mpsc::Sender<(StoreRequest, tokio::sync::oneshot::Sender<StoreResponse>)>),
}
#[automatically_derived]
#[allow(clippy::large_enum_variant)]
impl ::core::fmt::Debug for StoreClientBackend {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            StoreClientBackend::Grpc {
                client: __self_0,
                health: __self_1,
            } => ::core::fmt::Formatter::debug_struct_field2_finish(
                f, "Grpc", "client", &__self_0, "health", &__self_1,
            ),
            StoreClientBackend::Mem(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Mem", &__self_0)
            }
        }
    }
}
#[automatically_derived]
#[allow(clippy::large_enum_variant)]
impl ::core::clone::Clone for StoreClientBackend {
    #[inline]
    fn clone(&self) -> StoreClientBackend {
        match self {
            StoreClientBackend::Grpc {
                client: __self_0,
                health: __self_1,
            } => StoreClientBackend::Grpc {
                client: ::core::clone::Clone::clone(__self_0),
                health: ::core::clone::Clone::clone(__self_1),
            },
            StoreClientBackend::Mem(__self_0) => {
                StoreClientBackend::Mem(::core::clone::Clone::clone(__self_0))
            }
        }
    }
}
#[allow(non_camel_case_types)]
pub enum StoreRequest {
    version(()),
    put(PutRequest),
    get(GetRequest),
    has(HasRequest),
    get_links(GetLinksRequest),
    get_size(GetSizeRequest),
}
#[allow(non_camel_case_types)]
pub enum StoreResponse {
    version(Result<VersionResponse, String>),
    put(Result<(), String>),
    get(Result<GetResponse, String>),
    has(Result<HasResponse, String>),
    get_links(Result<GetLinksResponse, String>),
    get_size(Result<GetSizeResponse, String>),
}
pub trait Store: Send + Sync + 'static {
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn version<'life0, 'async_trait>(
        &'life0 self,
        request: (),
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<VersionResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn put<'life0, 'async_trait>(
        &'life0 self,
        request: PutRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<()>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn get<'life0, 'async_trait>(
        &'life0 self,
        request: GetRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<GetResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn has<'life0, 'async_trait>(
        &'life0 self,
        request: HasRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<HasResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn get_links<'life0, 'async_trait>(
        &'life0 self,
        request: GetLinksRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<GetLinksResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn get_size<'life0, 'async_trait>(
        &'life0 self,
        request: GetSizeRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<GetSizeResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait;
}
impl Store for StoreClientBackend {
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn version<'life0, 'async_trait>(
        &'life0 self,
        req: (),
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<VersionResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) =
                ::core::option::Option::None::<anyhow::Result<VersionResponse>>
            {
                return __ret;
            }
            let __self = self;
            let req = req;
            let __ret: anyhow::Result<VersionResponse> = {
                match __self {
                    #[cfg(feature = "grpc")]
                    Self::Grpc { client, .. } => {
                        let req = iroh_metrics::req::trace_tonic_req(req);
                        let mut c = client.clone();
                        let res = store_client::StoreClient::version(&mut c, req).await?;
                        let res = res.into_inner();
                        Ok(res)
                    }
                    #[cfg(feature = "mem")]
                    Self::Mem(s) => {
                        let (s_res, r_res) = tokio::sync::oneshot::channel();
                        s.send((StoreRequest::version(req), s_res))
                            .await
                            .map_err(|_| {
                                ::anyhow::__private::must_use({
                                    let error = ::anyhow::__private::format_err(
                                        ::core::fmt::Arguments::new_v1(&["send failed"], &[]),
                                    );
                                    error
                                })
                            })?;
                        let res = r_res.await?;
                        #[allow(irrefutable_let_patterns)]
                        if let StoreResponse::version(res) = res {
                            return res.map_err(|e| {
                                ::anyhow::__private::must_use({
                                    use ::anyhow::__private::kind::*;
                                    let error = match e {
                                        error => (&error).anyhow_kind().new(error),
                                    };
                                    error
                                })
                            });
                        } else {
                            return ::anyhow::__private::Err({
                                let error = ::anyhow::__private::format_err(
                                    ::core::fmt::Arguments::new_v1(&["invalid response"], &[]),
                                );
                                error
                            });
                        }
                    }
                }
            };
            #[allow(unreachable_code)]
            __ret
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn put<'life0, 'async_trait>(
        &'life0 self,
        req: PutRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<()>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) =
                ::core::option::Option::None::<anyhow::Result<()>>
            {
                return __ret;
            }
            let __self = self;
            let req = req;
            let __ret: anyhow::Result<()> = {
                match __self {
                    #[cfg(feature = "grpc")]
                    Self::Grpc { client, .. } => {
                        let req = iroh_metrics::req::trace_tonic_req(req);
                        let mut c = client.clone();
                        let res = store_client::StoreClient::put(&mut c, req).await?;
                        let res = res.into_inner();
                        Ok(res)
                    }
                    #[cfg(feature = "mem")]
                    Self::Mem(s) => {
                        let (s_res, r_res) = tokio::sync::oneshot::channel();
                        s.send((StoreRequest::put(req), s_res)).await.map_err(|_| {
                            ::anyhow::__private::must_use({
                                let error = ::anyhow::__private::format_err(
                                    ::core::fmt::Arguments::new_v1(&["send failed"], &[]),
                                );
                                error
                            })
                        })?;
                        let res = r_res.await?;
                        #[allow(irrefutable_let_patterns)]
                        if let StoreResponse::put(res) = res {
                            return res.map_err(|e| {
                                ::anyhow::__private::must_use({
                                    use ::anyhow::__private::kind::*;
                                    let error = match e {
                                        error => (&error).anyhow_kind().new(error),
                                    };
                                    error
                                })
                            });
                        } else {
                            return ::anyhow::__private::Err({
                                let error = ::anyhow::__private::format_err(
                                    ::core::fmt::Arguments::new_v1(&["invalid response"], &[]),
                                );
                                error
                            });
                        }
                    }
                }
            };
            #[allow(unreachable_code)]
            __ret
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn get<'life0, 'async_trait>(
        &'life0 self,
        req: GetRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<GetResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) =
                ::core::option::Option::None::<anyhow::Result<GetResponse>>
            {
                return __ret;
            }
            let __self = self;
            let req = req;
            let __ret: anyhow::Result<GetResponse> = {
                match __self {
                    #[cfg(feature = "grpc")]
                    Self::Grpc { client, .. } => {
                        let req = iroh_metrics::req::trace_tonic_req(req);
                        let mut c = client.clone();
                        let res = store_client::StoreClient::get(&mut c, req).await?;
                        let res = res.into_inner();
                        Ok(res)
                    }
                    #[cfg(feature = "mem")]
                    Self::Mem(s) => {
                        let (s_res, r_res) = tokio::sync::oneshot::channel();
                        s.send((StoreRequest::get(req), s_res)).await.map_err(|_| {
                            ::anyhow::__private::must_use({
                                let error = ::anyhow::__private::format_err(
                                    ::core::fmt::Arguments::new_v1(&["send failed"], &[]),
                                );
                                error
                            })
                        })?;
                        let res = r_res.await?;
                        #[allow(irrefutable_let_patterns)]
                        if let StoreResponse::get(res) = res {
                            return res.map_err(|e| {
                                ::anyhow::__private::must_use({
                                    use ::anyhow::__private::kind::*;
                                    let error = match e {
                                        error => (&error).anyhow_kind().new(error),
                                    };
                                    error
                                })
                            });
                        } else {
                            return ::anyhow::__private::Err({
                                let error = ::anyhow::__private::format_err(
                                    ::core::fmt::Arguments::new_v1(&["invalid response"], &[]),
                                );
                                error
                            });
                        }
                    }
                }
            };
            #[allow(unreachable_code)]
            __ret
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn has<'life0, 'async_trait>(
        &'life0 self,
        req: HasRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<HasResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) =
                ::core::option::Option::None::<anyhow::Result<HasResponse>>
            {
                return __ret;
            }
            let __self = self;
            let req = req;
            let __ret: anyhow::Result<HasResponse> = {
                match __self {
                    #[cfg(feature = "grpc")]
                    Self::Grpc { client, .. } => {
                        let req = iroh_metrics::req::trace_tonic_req(req);
                        let mut c = client.clone();
                        let res = store_client::StoreClient::has(&mut c, req).await?;
                        let res = res.into_inner();
                        Ok(res)
                    }
                    #[cfg(feature = "mem")]
                    Self::Mem(s) => {
                        let (s_res, r_res) = tokio::sync::oneshot::channel();
                        s.send((StoreRequest::has(req), s_res)).await.map_err(|_| {
                            ::anyhow::__private::must_use({
                                let error = ::anyhow::__private::format_err(
                                    ::core::fmt::Arguments::new_v1(&["send failed"], &[]),
                                );
                                error
                            })
                        })?;
                        let res = r_res.await?;
                        #[allow(irrefutable_let_patterns)]
                        if let StoreResponse::has(res) = res {
                            return res.map_err(|e| {
                                ::anyhow::__private::must_use({
                                    use ::anyhow::__private::kind::*;
                                    let error = match e {
                                        error => (&error).anyhow_kind().new(error),
                                    };
                                    error
                                })
                            });
                        } else {
                            return ::anyhow::__private::Err({
                                let error = ::anyhow::__private::format_err(
                                    ::core::fmt::Arguments::new_v1(&["invalid response"], &[]),
                                );
                                error
                            });
                        }
                    }
                }
            };
            #[allow(unreachable_code)]
            __ret
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn get_links<'life0, 'async_trait>(
        &'life0 self,
        req: GetLinksRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<GetLinksResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) =
                ::core::option::Option::None::<anyhow::Result<GetLinksResponse>>
            {
                return __ret;
            }
            let __self = self;
            let req = req;
            let __ret: anyhow::Result<GetLinksResponse> = {
                match __self {
                    #[cfg(feature = "grpc")]
                    Self::Grpc { client, .. } => {
                        let req = iroh_metrics::req::trace_tonic_req(req);
                        let mut c = client.clone();
                        let res = store_client::StoreClient::get_links(&mut c, req).await?;
                        let res = res.into_inner();
                        Ok(res)
                    }
                    #[cfg(feature = "mem")]
                    Self::Mem(s) => {
                        let (s_res, r_res) = tokio::sync::oneshot::channel();
                        s.send((StoreRequest::get_links(req), s_res))
                            .await
                            .map_err(|_| {
                                ::anyhow::__private::must_use({
                                    let error = ::anyhow::__private::format_err(
                                        ::core::fmt::Arguments::new_v1(&["send failed"], &[]),
                                    );
                                    error
                                })
                            })?;
                        let res = r_res.await?;
                        #[allow(irrefutable_let_patterns)]
                        if let StoreResponse::get_links(res) = res {
                            return res.map_err(|e| {
                                ::anyhow::__private::must_use({
                                    use ::anyhow::__private::kind::*;
                                    let error = match e {
                                        error => (&error).anyhow_kind().new(error),
                                    };
                                    error
                                })
                            });
                        } else {
                            return ::anyhow::__private::Err({
                                let error = ::anyhow::__private::format_err(
                                    ::core::fmt::Arguments::new_v1(&["invalid response"], &[]),
                                );
                                error
                            });
                        }
                    }
                }
            };
            #[allow(unreachable_code)]
            __ret
        })
    }
    #[allow(
        clippy::let_unit_value,
        clippy::no_effect_underscore_binding,
        clippy::shadow_same,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn get_size<'life0, 'async_trait>(
        &'life0 self,
        req: GetSizeRequest,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = anyhow::Result<GetSizeResponse>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret) =
                ::core::option::Option::None::<anyhow::Result<GetSizeResponse>>
            {
                return __ret;
            }
            let __self = self;
            let req = req;
            let __ret: anyhow::Result<GetSizeResponse> = {
                match __self {
                    #[cfg(feature = "grpc")]
                    Self::Grpc { client, .. } => {
                        let req = iroh_metrics::req::trace_tonic_req(req);
                        let mut c = client.clone();
                        let res = store_client::StoreClient::get_size(&mut c, req).await?;
                        let res = res.into_inner();
                        Ok(res)
                    }
                    #[cfg(feature = "mem")]
                    Self::Mem(s) => {
                        let (s_res, r_res) = tokio::sync::oneshot::channel();
                        s.send((StoreRequest::get_size(req), s_res))
                            .await
                            .map_err(|_| {
                                ::anyhow::__private::must_use({
                                    let error = ::anyhow::__private::format_err(
                                        ::core::fmt::Arguments::new_v1(&["send failed"], &[]),
                                    );
                                    error
                                })
                            })?;
                        let res = r_res.await?;
                        #[allow(irrefutable_let_patterns)]
                        if let StoreResponse::get_size(res) = res {
                            return res.map_err(|e| {
                                ::anyhow::__private::must_use({
                                    use ::anyhow::__private::kind::*;
                                    let error = match e {
                                        error => (&error).anyhow_kind().new(error),
                                    };
                                    error
                                })
                            });
                        } else {
                            return ::anyhow::__private::Err({
                                let error = ::anyhow::__private::format_err(
                                    ::core::fmt::Arguments::new_v1(&["invalid response"], &[]),
                                );
                                error
                            });
                        }
                    }
                }
            };
            #[allow(unreachable_code)]
            __ret
        })
    }
}
#[cfg(feature = "grpc")]
mod grpc {
    use super::*;
    use tonic::{Request, Response, Status};
    impl<P: Store> store_server::Store for P {
        #[allow(
            clippy::let_unit_value,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn version<'life0, 'async_trait>(
            &'life0 self,
            req: Request<()>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = Result<Response<VersionResponse>, Status>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) =
                    ::core::option::Option::None::<Result<Response<VersionResponse>, Status>>
                {
                    return __ret;
                }
                let __self = self;
                let req = req;
                let __ret: Result<Response<VersionResponse>, Status> = {
                    let req = req.into_inner();
                    let res = Store::version(__self, req)
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    Ok(Response::new(res))
                };
                #[allow(unreachable_code)]
                __ret
            })
        }
        #[allow(
            clippy::let_unit_value,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn put<'life0, 'async_trait>(
            &'life0 self,
            req: Request<PutRequest>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = Result<Response<()>, Status>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) =
                    ::core::option::Option::None::<Result<Response<()>, Status>>
                {
                    return __ret;
                }
                let __self = self;
                let req = req;
                let __ret: Result<Response<()>, Status> = {
                    let req = req.into_inner();
                    let res = Store::put(__self, req)
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    Ok(Response::new(res))
                };
                #[allow(unreachable_code)]
                __ret
            })
        }

        fn put_many<'life0, 'async_trait>(
            &'life0 self,
            req: Request<PutRequest>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = Result<Response<()>, Status>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) =
                    ::core::option::Option::None::<Result<Response<()>, Status>>
                {
                    return __ret;
                }
                let __self = self;
                let req = req;
                let __ret: Result<Response<()>, Status> = {
                    let req = req.into_inner();
                    let res = Store::put(__self, req)
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    Ok(Response::new(res))
                };
                #[allow(unreachable_code)]
                __ret
            })
        }
        #[allow(
            clippy::let_unit_value,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn get<'life0, 'async_trait>(
            &'life0 self,
            req: Request<GetRequest>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = Result<Response<GetResponse>, Status>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) =
                    ::core::option::Option::None::<Result<Response<GetResponse>, Status>>
                {
                    return __ret;
                }
                let __self = self;
                let req = req;
                let __ret: Result<Response<GetResponse>, Status> = {
                    let req = req.into_inner();
                    let res = Store::get(__self, req)
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    Ok(Response::new(res))
                };
                #[allow(unreachable_code)]
                __ret
            })
        }
        #[allow(
            clippy::let_unit_value,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn has<'life0, 'async_trait>(
            &'life0 self,
            req: Request<HasRequest>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = Result<Response<HasResponse>, Status>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) =
                    ::core::option::Option::None::<Result<Response<HasResponse>, Status>>
                {
                    return __ret;
                }
                let __self = self;
                let req = req;
                let __ret: Result<Response<HasResponse>, Status> = {
                    let req = req.into_inner();
                    let res = Store::has(__self, req)
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    Ok(Response::new(res))
                };
                #[allow(unreachable_code)]
                __ret
            })
        }
        #[allow(
            clippy::let_unit_value,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn get_links<'life0, 'async_trait>(
            &'life0 self,
            req: Request<GetLinksRequest>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = Result<Response<GetLinksResponse>, Status>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) =
                    ::core::option::Option::None::<Result<Response<GetLinksResponse>, Status>>
                {
                    return __ret;
                }
                let __self = self;
                let req = req;
                let __ret: Result<Response<GetLinksResponse>, Status> = {
                    let req = req.into_inner();
                    let res = Store::get_links(__self, req)
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    Ok(Response::new(res))
                };
                #[allow(unreachable_code)]
                __ret
            })
        }
        #[allow(
            clippy::let_unit_value,
            clippy::no_effect_underscore_binding,
            clippy::shadow_same,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn get_size<'life0, 'async_trait>(
            &'life0 self,
            req: Request<GetSizeRequest>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<Output = Result<Response<GetSizeResponse>, Status>>
                    + ::core::marker::Send
                    + 'async_trait,
            >,
        >
        where
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                if let ::core::option::Option::Some(__ret) =
                    ::core::option::Option::None::<Result<Response<GetSizeResponse>, Status>>
                {
                    return __ret;
                }
                let __self = self;
                let req = req;
                let __ret: Result<Response<GetSizeResponse>, Status> = {
                    let req = req.into_inner();
                    let res = Store::get_size(__self, req)
                        .await
                        .map_err(|err| Status::internal(err.to_string()))?;
                    Ok(Response::new(res))
                };
                #[allow(unreachable_code)]
                __ret
            })
        }
    }
}
