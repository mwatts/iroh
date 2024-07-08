use std::rc::Rc;

use futures_concurrency::stream::StreamExt as _;
use futures_lite::StreamExt as _;
use genawaiter::GeneratorState;
use strum::IntoEnumIterator;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error_span, trace, warn, Span};

use crate::{
    auth::InterestMap,
    proto::sync::{ControlIssueGuarantee, InitialTransmission, LogicalChannel, Message},
    session::{
        aoi_finder::AoiFinder,
        capabilities::Capabilities,
        channels::{ChannelSenders, LogicalChannelReceivers},
        events::{Event, EventEmitter},
        pai_finder::{self as pai, PaiFinder, PaiIntersection},
        static_tokens::StaticTokens,
        Channels, Error, Role, SessionId, SessionInit,
    },
    store::{
        traits::{SecretStorage, Storage},
        Store,
    },
    util::{channel::Receiver, stream::Cancelable, task::SharedJoinMap},
};

use super::{
    channels::ChannelReceivers,
    data::{DataReceiver, DataSender},
    reconciler::Reconciler,
    SessionMode,
};

const INITIAL_GUARANTEES: u64 = u64::MAX;

pub async fn run_session<S: Storage>(
    store: Store<S>,
    channels: Channels,
    cancel_token: CancellationToken,
    session_id: SessionId,
    our_role: Role,
    init: SessionInit,
    initial_transmission: InitialTransmission,
) -> Result<(), Error> {
    let Channels { send, recv } = channels;
    let ChannelReceivers {
        control_recv,
        logical_recv:
            LogicalChannelReceivers {
                reconciliation_recv,
                static_tokens_recv,
                capability_recv,
                aoi_recv,
                data_recv,
                intersection_recv,
            },
    } = recv;

    // Make all our receivers close once the cancel_token is triggered.
    let control_recv = Cancelable::new(control_recv, cancel_token.clone());
    let reconciliation_recv = Cancelable::new(reconciliation_recv, cancel_token.clone());
    let intersection_recv = Cancelable::new(intersection_recv, cancel_token.clone());
    let mut static_tokens_recv = Cancelable::new(static_tokens_recv, cancel_token.clone());
    let mut capability_recv = Cancelable::new(capability_recv, cancel_token.clone());
    let mut aoi_recv = Cancelable::new(aoi_recv, cancel_token.clone());
    let mut data_recv = Cancelable::new(data_recv, cancel_token.clone());

    let events = EventEmitter::default();

    let caps = Capabilities::new(
        initial_transmission.our_nonce,
        initial_transmission.received_commitment,
    );
    let tokens = StaticTokens::default();
    let aoi_finder = AoiFinder::default();

    let tasks = Tasks::default();

    let interests = store.auth().find_read_caps_for_interests(init.interests)?;
    let interests = Rc::new(interests);

    // Setup the private area intersection finder.
    let (pai_inbox_tx, pai_inbox_rx) = flume::bounded(128);
    tasks.spawn(error_span!("pai"), {
        let store = store.clone();
        let send = send.clone();
        let caps = caps.clone();
        let inbox = pai_inbox_rx
            .into_stream()
            .merge(intersection_recv.map(pai::Input::ReceivedMessage));
        let interests = Rc::clone(&interests);
        let aoi_finder = aoi_finder.clone();
        let events = events.clone();
        async move {
            let mut gen = PaiFinder::run_gen(inbox);
            loop {
                match gen.async_resume().await {
                    GeneratorState::Yielded(output) => match output {
                        pai::Output::SendMessage(message) => send.send(message).await?,
                        pai::Output::NewIntersection(intersection) => {
                            events
                                .send(Event::CapabilityIntersection(
                                    intersection.authorisation.clone(),
                                ))
                                .await?;
                            on_pai_intersection(
                                &interests,
                                store.secrets(),
                                &aoi_finder,
                                &caps,
                                &send,
                                intersection,
                            )
                            .await?;
                        }
                        pai::Output::SignAndSendSubspaceCap(handle, cap) => {
                            let message =
                                caps.sign_subspace_capabiltiy(store.secrets(), cap, handle)?;
                            send.send(Box::new(message)).await?;
                        }
                    },
                    GeneratorState::Complete(res) => {
                        return res;
                    }
                }
            }
        }
    });

    // Spawn a task to handle incoming static tokens.
    tasks.spawn(error_span!("stt"), {
        let tokens = tokens.clone();
        async move {
            while let Some(message) = static_tokens_recv.try_next().await? {
                tokens.bind_theirs(message.static_token);
            }
            Ok(())
        }
    });

    // Only setup data receiver if session is configured in live mode.
    if init.mode == SessionMode::Live {
        tasks.spawn(error_span!("data-recv"), {
            let store = store.clone();
            let tokens = tokens.clone();
            async move {
                let mut data_receiver = DataReceiver::new(store, tokens, session_id);
                while let Some(message) = data_recv.try_next().await? {
                    data_receiver.on_message(message).await?;
                }
                Ok(())
            }
        });
        tasks.spawn(error_span!("data-send"), {
            let store = store.clone();
            let tokens = tokens.clone();
            let send = send.clone();
            let aoi_intersections = aoi_finder.subscribe();
            async move {
                DataSender::new(store, send, aoi_intersections, tokens, session_id)
                    .run()
                    .await?;
                Ok(())
            }
        });
    }

    // Spawn a task to handle incoming capabilities.
    tasks.spawn(error_span!("cap-recv"), {
        let to_pai = pai_inbox_tx.clone();
        let caps = caps.clone();
        async move {
            while let Some(message) = capability_recv.try_next().await? {
                let handle = message.handle;
                caps.validate_and_bind_theirs(message.capability, message.signature)?;
                to_pai
                    .send_async(pai::Input::ReceivedReadCapForIntersection(handle))
                    .await
                    .map_err(|_| Error::InvalidState("PAI actor dead"))?;
            }
            Ok(())
        }
    });

    // Spawn a task to handle incoming areas of interest.
    tasks.spawn(error_span!("aoi-recv"), {
        let aoi_finder = aoi_finder.clone();
        let caps = caps.clone();
        async move {
            while let Some(message) = aoi_recv.try_next().await? {
                let cap = caps.get_theirs_eventually(message.authorisation).await;
                aoi_finder.validate_and_bind_theirs(&cap, message.area_of_interest)?;
            }
            aoi_finder.close();
            Ok(())
        }
    });

    // Spawn a task to handle reconciliation messages
    tasks.spawn(error_span!("rec"), {
        let cancel_token = cancel_token.clone();
        let aoi_intersections = aoi_finder.subscribe();
        let reconciler = Reconciler::new(
            store.clone(),
            reconciliation_recv,
            aoi_intersections,
            tokens.clone(),
            session_id,
            send.clone(),
            our_role,
            events,
        )?;
        async move {
            let res = reconciler.run().await;
            if res.is_ok() && !init.mode.is_live() {
                debug!("reconciliation complete and not in live mode: trigger cancel");
                cancel_token.cancel();
            }
            res
        }
    });

    // Spawn a task to handle control messages
    tasks.spawn(error_span!("ctl-recv"), {
        let cancel_token = cancel_token.clone();
        let fut = control_loop(
            our_role,
            interests,
            caps,
            send.clone(),
            tasks.clone(),
            control_recv,
            pai_inbox_tx,
        );
        async move {
            let res = fut.await;
            if res.is_ok() {
                debug!("control channel closed: trigger cancel");
                cancel_token.cancel();
            }
            res
        }
    });

    // Wait until the session is cancelled, or until a task fails.
    let result = loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("cancel token triggered: close session");
                break Ok(());
            },
            Some((span, result)) = tasks.join_next() => {
                let _guard = span.enter();
                trace!(?result, remaining = tasks.remaining_tasks(), "task complete");
                match result {
                    Err(err) => {
                        warn!(?err, "session task paniced: abort session");
                        break Err(Error::TaskFailed(err));
                    },
                    Ok(Err(err)) => {
                        warn!(?err, "session task failed: abort session");
                        break Err(err);
                    }
                    Ok(Ok(())) => {}
                }
            },
        }
    };

    if result.is_err() {
        debug!("aborting session");
        tasks.abort_all();
    } else {
        debug!("closing session");
    }

    // Unsubscribe from the store.  This stops the data send task.
    store.entries().unsubscribe(&session_id);

    // Wait for remaining tasks to terminate to catch any panics.
    // TODO: Add timeout?
    while let Some((span, result)) = tasks.join_next().await {
        let _guard = span.enter();
        trace!(
            ?result,
            remaining = tasks.remaining_tasks(),
            "task complete"
        );
        match result {
            Err(err) if err.is_cancelled() => {}
            Err(err) => warn!("task paniced: {err:?}"),
            Ok(Err(err)) => warn!("task failed: {err:?}"),
            Ok(Ok(())) => {}
        }
    }

    // Close our channel senders.
    // This will stop the network send loop after all pending data has been sent.
    send.close_all();

    debug!(success = result.is_ok(), "session complete");
    result
}

pub type Tasks = SharedJoinMap<Span, Result<(), Error>>;

async fn control_loop(
    our_role: Role,
    our_interests: Rc<InterestMap>,
    caps: Capabilities,
    sender: ChannelSenders,
    tasks: Tasks,
    mut control_recv: Cancelable<Receiver<Message>>,
    to_pai: flume::Sender<pai::Input>,
) -> Result<(), Error> {
    debug!(role = ?our_role, "start session");
    // Reveal our nonce.
    let reveal_message = caps.reveal_commitment()?;
    sender.send(reveal_message).await?;

    // Issue guarantees for all logical channels.
    for channel in LogicalChannel::iter() {
        let msg = ControlIssueGuarantee {
            amount: INITIAL_GUARANTEES,
            channel,
        };
        sender.send(msg).await?;
    }

    // Handle incoming messages on the control channel.
    while let Some(message) = control_recv.try_next().await? {
        match message {
            Message::CommitmentReveal(msg) => {
                caps.received_commitment_reveal(our_role, msg.nonce)?;

                let submit_interests_fut = {
                    let to_pai = to_pai.clone();
                    let our_interests = Rc::clone(&our_interests);
                    async move {
                        for authorisation in our_interests.keys() {
                            to_pai
                                .send_async(pai::Input::SubmitAuthorisation(authorisation.clone()))
                                .await
                                .map_err(|_| Error::InvalidState("PAI actor dead"))?;
                        }
                        Ok(())
                    }
                };
                tasks.spawn(error_span!("setup-pai"), submit_interests_fut);
            }
            Message::ControlIssueGuarantee(msg) => {
                let ControlIssueGuarantee { amount, channel } = msg;
                // trace!(?channel, %amount, "add guarantees");
                sender.get_logical(channel).add_guarantees(amount);
            }
            Message::PaiRequestSubspaceCapability(msg) => {
                to_pai
                    .send_async(pai::Input::ReceivedSubspaceCapRequest(msg.handle))
                    .await
                    .map_err(|_| Error::InvalidState("PAI actor dead"))?;
            }
            Message::PaiReplySubspaceCapability(msg) => {
                caps.verify_subspace_cap(&msg.capability, &msg.signature)?;
                to_pai
                    .send_async(pai::Input::ReceivedVerifiedSubspaceCapReply(
                        msg.handle,
                        msg.capability.granted_namespace().id(),
                    ))
                    .await
                    .map_err(|_| Error::InvalidState("PAI actor dead"))?;
            }
            _ => return Err(Error::UnsupportedMessage),
        }
    }

    Ok(())
}

async fn on_pai_intersection<S: SecretStorage>(
    interests: &InterestMap,
    secrets: &S,
    aoi_finder: &AoiFinder,
    capabilities: &Capabilities,
    sender: &ChannelSenders,
    intersection: PaiIntersection,
) -> Result<(), Error> {
    let PaiIntersection {
        authorisation,
        handle,
    } = intersection;
    let aois = interests
        .get(&authorisation)
        .ok_or(Error::NoKnownInterestsForCapability)?;
    let namespace = authorisation.namespace();
    let capability_handle = capabilities
        .bind_and_send_ours(secrets, sender, handle, authorisation.read_cap().clone())
        .await?;

    for aoi in aois.iter().cloned() {
        aoi_finder
            .bind_and_send_ours(sender, namespace, aoi, capability_handle)
            .await?;
    }
    Ok(())
}