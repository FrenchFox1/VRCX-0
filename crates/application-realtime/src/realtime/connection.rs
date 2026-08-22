use std::any::Any;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::Arc;

use futures_util::FutureExt;
use tokio::sync::watch;

use super::{
    RealtimeSessionContext, RealtimeTransportTermination, RealtimeWsMessagePayload,
    RealtimeWsStatus,
};

pub type RealtimeTransportFuture =
    Pin<Box<dyn Future<Output = RealtimeTransportTermination> + Send + 'static>>;

pub trait RealtimeMessageSink: Send + Sync {
    fn handle_realtime_transport_status(
        &self,
        _generation: u64,
        _session_generation: u64,
        _session: &RealtimeSessionContext,
        _status: RealtimeWsStatus,
    ) {
    }

    fn handle_realtime_ws_message(
        &self,
        generation: u64,
        session_generation: u64,
        session: &RealtimeSessionContext,
        payload: &RealtimeWsMessagePayload,
    );
}

pub trait RealtimeTransport: Send + Sync {
    fn run(
        &self,
        message_sink: Arc<dyn RealtimeMessageSink>,
        client_run_id: u64,
        generation: u64,
        session_generation: u64,
        session: RealtimeSessionContext,
        cancel_rx: watch::Receiver<u64>,
    ) -> RealtimeTransportFuture;
}

pub(super) async fn supervise_realtime_transport<F>(transport: F) -> RealtimeTransportTermination
where
    F: Future<Output = RealtimeTransportTermination>,
{
    match AssertUnwindSafe(transport).catch_unwind().await {
        Ok(termination) => termination,
        Err(payload) => RealtimeTransportTermination::UnexpectedExit {
            reason: panic_reason(payload),
            connected_secs: None,
        },
    }
}

fn panic_reason(payload: Box<dyn Any + Send>) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|reason| (*reason).to_string())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "Realtime transport panicked without an error message.".into())
}

#[cfg(test)]
mod tests {
    use super::supervise_realtime_transport;
    use crate::realtime::RealtimeTransportTermination;

    #[tokio::test]
    async fn transport_panics_are_converted_to_unexpected_exit() {
        let termination = supervise_realtime_transport(async {
            panic!("transport panic");
            #[allow(unreachable_code)]
            RealtimeTransportTermination::Stopped
        })
        .await;

        assert!(matches!(
            termination,
            RealtimeTransportTermination::UnexpectedExit { reason, .. }
                if reason == "transport panic"
        ));
    }
}
