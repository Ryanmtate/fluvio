//!
//! # Service Implementataion
//!
//! Public service API allows 3rd party systems to invoke operations on Fluvio
//! Streaming Controller. Requests are received and dispatched to handlers
//! based on API keys.
//!

use std::sync::Arc;
use std::marker::PhantomData;

use async_trait::async_trait;
use futures::io::AsyncRead;
use futures::io::AsyncWrite;

use kf_service::api_loop;
use kf_service::call_service;
use kf_socket::InnerKfSocket;
use kf_socket::KfSocketError;
use kf_service::KfService;
use sc_api::PublicRequest;
use sc_api::ScApiKey;
use k8_metadata_client::MetadataClient;
use flv_future_aio::zero_copy::ZeroCopyWrite;

use super::api::handle_api_versions_request;
use super::api::handle_kf_metadata_request;

use super::api::handle_create_topics_request;
use super::api::handle_delete_topics_request;
use super::api::handle_fetch_topics_request;
use super::api::handle_topic_composition_request;
use super::api::handle_create_custom_spus_request;
use super::api::handle_delete_custom_spus_request;
use super::api::handle_fetch_spu_request;
use super::api::handle_create_spu_groups_request;
use super::api::handle_delete_spu_groups_request;
use super::api::handle_fetch_spu_groups_request;

use super::SharedPublicContext;

pub struct PublicService<C>(PhantomData<C>);

impl<C> PublicService<C> {
    pub fn new() -> Self {
        PublicService(PhantomData)
    }
}

#[async_trait]
impl<C,S> KfService<S> for PublicService<C>
where
    C: MetadataClient,
    S: AsyncWrite + AsyncRead + Unpin + Send + ZeroCopyWrite +'static
{
    type Context = SharedPublicContext<C>;
    type Request = PublicRequest;

    async fn respond(
        self: Arc<Self>,
        ctx: Self::Context,
        socket: InnerKfSocket<S>,
    ) -> Result<(), KfSocketError> {
        let (mut sink, mut stream) = socket.split();
        let mut api_stream = stream.api_stream::<PublicRequest, ScApiKey>();

        api_loop!(
            api_stream,

            // Common
            PublicRequest::ApiVersionsRequest(request) => call_service!(
                request,
                handle_api_versions_request(request),
                sink,
                "api version handler"
            ),

            // Kafka
            PublicRequest::KfMetadataRequest(request) => call_service!(
                request,
                handle_kf_metadata_request(request, ctx.metadata.clone()),
                sink,
                "metadata request handler"
            ),

            // Fluvio - Topics
            PublicRequest::FlvCreateTopicsRequest(request) => call_service!(
                request,
                handle_create_topics_request(request, &ctx),
                sink,
                "create topic handler"
            ),
            PublicRequest::FlvDeleteTopicsRequest(request) => call_service!(
                request,
                handle_delete_topics_request(request, &ctx),
                sink,
                "delete topic handler"
            ),
            PublicRequest::FlvFetchTopicsRequest(request) => call_service!(
                request,
                handle_fetch_topics_request(request, ctx.metadata.clone()),
                sink,
                "fetch topic handler"
            ),
            PublicRequest::FlvTopicCompositionRequest(request) => call_service!(
                request,
                handle_topic_composition_request(request, ctx.metadata.clone()),
                sink,
                "topic metadata handler"
            ),

            // Fluvio - Spus
            PublicRequest::FlvCreateCustomSpusRequest(request) => call_service!(
                request,
                handle_create_custom_spus_request(request, &ctx),
                sink,
                "create custom spus handler"
            ),
            PublicRequest::FlvDeleteCustomSpusRequest(request) => call_service!(
                request,
                handle_delete_custom_spus_request(request, &ctx),
                sink,
                "delete custom spus handler"
            ),
            PublicRequest::FlvFetchSpusRequest(request) => call_service!(
                request,
                handle_fetch_spu_request(request, ctx.metadata.clone()),
                sink,
                "fetch spus handler"
            ),

            PublicRequest::FlvCreateSpuGroupsRequest(request) => call_service!(
                request,
                handle_create_spu_groups_request(request, &ctx),
                sink,
                "create spu groups handler"
            ),
            PublicRequest::FlvDeleteSpuGroupsRequest(request) => call_service!(
                request,
                handle_delete_spu_groups_request(request, &ctx),
                sink,
                "delete spu groups handler"
            ),
            PublicRequest::FlvFetchSpuGroupsRequest(request) => call_service!(
                request,
                handle_fetch_spu_groups_request(request, &ctx),
                sink,
                "fetch spu groups handler"
            )

        );

        Ok(())
    }
}
