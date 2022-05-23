/// StdioData is a single chunk of stdout or stderr data that is streamed
/// from GRPCStdio.
#[derive(serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StdioData {
    #[prost(enumeration="stdio_data::Channel", tag="1")]
    pub channel: i32,
    #[prost(bytes="vec", tag="2")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
/// Nested message and enum types in `StdioData`.
pub mod stdio_data {
    #[derive(serde::Deserialize)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Channel {
        Invalid = 0,
        Stdout = 1,
        Stderr = 2,
    }
}
/// Generated client implementations.
pub mod grpc_stdio_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// GRPCStdio is a service that is automatically run by the plugin process
    /// to stream any stdout/err data so that it can be mirrored on the plugin
    /// host side.
    #[derive(Debug, Clone)]
    pub struct GrpcStdioClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GrpcStdioClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> GrpcStdioClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> GrpcStdioClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            GrpcStdioClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// StreamStdio returns a stream that contains all the stdout/stderr.
        /// This RPC endpoint must only be called ONCE. Once stdio data is consumed
        /// it is not sent again.
        ///
        /// Callers should connect early to prevent blocking on the plugin process.
        pub async fn stream_stdio(
            &mut self,
            request: impl tonic::IntoRequest<super::super::google::protobuf::Empty>,
        ) -> Result<
                tonic::Response<tonic::codec::Streaming<super::StdioData>>,
                tonic::Status,
            > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/plugin.GRPCStdio/StreamStdio",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod grpc_stdio_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with GrpcStdioServer.
    #[async_trait]
    pub trait GrpcStdio: Send + Sync + 'static {
        ///Server streaming response type for the StreamStdio method.
        type StreamStdioStream: futures_core::Stream<
                Item = Result<super::StdioData, tonic::Status>,
            >
            + Send
            + 'static;
        /// StreamStdio returns a stream that contains all the stdout/stderr.
        /// This RPC endpoint must only be called ONCE. Once stdio data is consumed
        /// it is not sent again.
        ///
        /// Callers should connect early to prevent blocking on the plugin process.
        async fn stream_stdio(
            &self,
            request: tonic::Request<super::super::google::protobuf::Empty>,
        ) -> Result<tonic::Response<Self::StreamStdioStream>, tonic::Status>;
    }
    /// GRPCStdio is a service that is automatically run by the plugin process
    /// to stream any stdout/err data so that it can be mirrored on the plugin
    /// host side.
    #[derive(Debug)]
    pub struct GrpcStdioServer<T: GrpcStdio> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: GrpcStdio> GrpcStdioServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for GrpcStdioServer<T>
    where
        T: GrpcStdio,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/plugin.GRPCStdio/StreamStdio" => {
                    #[allow(non_camel_case_types)]
                    struct StreamStdioSvc<T: GrpcStdio>(pub Arc<T>);
                    impl<
                        T: GrpcStdio,
                    > tonic::server::ServerStreamingService<
                        super::super::google::protobuf::Empty,
                    > for StreamStdioSvc<T> {
                        type Response = super::StdioData;
                        type ResponseStream = T::StreamStdioStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                super::super::google::protobuf::Empty,
                            >,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).stream_stdio(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StreamStdioSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: GrpcStdio> Clone for GrpcStdioServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: GrpcStdio> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: GrpcStdio> tonic::transport::NamedService for GrpcStdioServer<T> {
        const NAME: &'static str = "plugin.GRPCStdio";
    }
}
