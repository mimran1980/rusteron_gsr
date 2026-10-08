//! A persistent subscription that follows its stream's newest recording.

use super::*;
use std::ffi::{CStr, CString};
use std::rc::Rc;

type ArchiveContextFactory = Box<dyn Fn() -> Result<AeronArchiveContext, AeronCError>>;
type RecordingFilter = Rc<dyn Fn(&AeronArchiveRecordingDescriptor) -> bool>;
type ListConsumer = Box<dyn FnMut(AeronArchiveRecordingDescriptor)>;

/// A persistent subscription that follows its stream's newest recording.
///
/// A restarted publisher records a new session, and a persistent subscription on the old
/// recording then fails. This one lists the archive's recordings again and continues with
/// the newest that [`Self::recordings`] accepts: from its start, or from the last message
/// delivered if it is still the recording it was following. It looks again when the
/// subscription fails, or sits off the live stream with nothing to read for
/// [`Self::idle_timeout`], as one does once its publisher's session has gone. The live
/// stream is taken only from the followed recording's session, never another publisher's.
/// No step blocks, and a failed lookup or subscription is retried after [`Self::retry_after`].
pub struct FollowingPersistentSubscription {
    aeron: Aeron,
    archive_context: ArchiveContextFactory,
    live_channel: String,
    stream_id: i32,
    replay_channel: String,
    replay_stream_id: i32,
    channel_fragment: CString,
    filter: RecordingFilter,
    from_start: bool,
    retry: Duration,
    idle_timeout: Duration,
    /// When the followed recording last delivered a fragment or was live.
    active_at: Instant,
    stage: Stage,
    /// The recording last subscribed to, and where to resume it.
    resume: Option<(i64, i64)>,
}

enum Stage {
    Waiting(Instant),
    Connecting(AeronArchiveAsyncConnect, AeronArchiveContext),
    Listing(
        AeronArchiveAsyncListRecordings<ListConsumer>,
        Rc<Cell<Option<(i64, i32)>>>,
    ),
    Building(AeronArchiveAsyncPersistentSubscription, i64),
    Following {
        subscription: AeronArchivePersistentSubscription,
        recording_id: i64,
    },
}

impl FollowingPersistentSubscription {
    /// Follows the recordings of `live`, a channel and stream id, replaying them to `replay`.
    /// `archive_context` makes a fresh context, with the archive's control channels, for
    /// each connection to the archive.
    pub fn new(
        aeron: &Aeron,
        archive_context: impl Fn() -> Result<AeronArchiveContext, AeronCError> + 'static,
        live: (&str, i32),
        replay: (&str, i32),
    ) -> Self {
        Self {
            aeron: aeron.clone(),
            archive_context: Box::new(archive_context),
            live_channel: live.0.to_owned(),
            stream_id: live.1,
            replay_channel: replay.0.to_owned(),
            replay_stream_id: replay.1,
            channel_fragment: CString::new(live.0).unwrap_or_default(),
            filter: Rc::new(|recording| recording.stop_position() == i64::from(AERON_NULL_VALUE)),
            from_start: false,
            retry: Duration::from_secs(1),
            idle_timeout: Duration::from_secs(5),
            active_at: Instant::now(),
            stage: Stage::Waiting(Instant::now()),
            resume: None,
        }
    }

    /// Follows only the recordings whose channel contains `channel_fragment` and that
    /// `filter` accepts. By default, the live channel's recordings still recording.
    #[must_use]
    pub fn recordings(
        mut self,
        channel_fragment: &CStr,
        filter: impl Fn(&AeronArchiveRecordingDescriptor) -> bool + 'static,
    ) -> Self {
        self.channel_fragment = channel_fragment.to_owned();
        self.filter = Rc::new(filter);
        self
    }

    /// Replays the first recording from its start instead of joining the live stream.
    #[must_use]
    pub fn from_start(mut self) -> Self {
        self.from_start = true;
        self
    }

    /// How long to wait before retrying a failed lookup or subscription; one second by default.
    #[must_use]
    pub fn retry_after(mut self, retry: Duration) -> Self {
        self.retry = retry;
        self
    }

    /// How long the followed recording may sit off the live stream with nothing to read
    /// before the archive is asked for a newer one; five seconds by default.
    #[must_use]
    pub fn idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.idle_timeout = idle_timeout;
        self
    }

    /// The recording being followed, `None` while the next is looked up.
    pub fn recording_id(&self) -> Option<i64> {
        match &self.stage {
            Stage::Following { recording_id, .. } => Some(*recording_id),
            _ => None,
        }
    }

    /// Taking the live stream rather than replaying.
    pub fn is_live(&self) -> bool {
        matches!(&self.stage, Stage::Following { subscription, .. } if subscription.is_live())
    }

    /// Polls the followed recording for up to `fragment_limit` fragments, or advances the
    /// lookup of the next one; returns the fragments read.
    pub fn poll_fn(&mut self, mut handler: impl FnMut(&[u8], AeronHeader), fragment_limit: usize) -> i32 {
        let Stage::Following {
            subscription,
            recording_id,
            ..
        } = &self.stage
        else {
            self.advance();
            return 0;
        };
        let (mut last, mut read) = (None, 0);
        // the subscription's own count includes work other than fragments
        let _ = subscription.poll_fn(
            |message, header| {
                last = Some(header.position());
                read += 1;
                handler(message, header);
            },
            fragment_limit,
        );
        let now = Instant::now();
        if let Some(position) = last {
            self.resume = Some((*recording_id, position));
        }
        if last.is_some() || subscription.is_live() {
            self.active_at = now;
        }
        if subscription.has_failed() {
            let why = subscription
                .get_failure_reason()
                .map_or_else(String::new, |(_, reason)| reason);
            log::info!("recording {recording_id} failed ({why}); looking for the next");
            self.stage = Stage::Waiting(now + self.retry);
        } else if now.duration_since(self.active_at) >= self.idle_timeout {
            log::info!("recording {recording_id} idle off the live stream; looking for the next");
            self.stage = Stage::Waiting(now);
        }
        read
    }

    fn advance(&mut self) {
        let stage = std::mem::replace(&mut self.stage, Stage::Waiting(Instant::now()));
        self.stage = self.step(stage).unwrap_or_else(|e| {
            log::warn!("following stream {}: {e}; retrying in {:?}", self.stream_id, self.retry);
            Stage::Waiting(Instant::now() + self.retry)
        });
    }

    fn step(&mut self, stage: Stage) -> Result<Stage, String> {
        Ok(match stage {
            Stage::Waiting(at) if Instant::now() < at => Stage::Waiting(at),
            Stage::Waiting(_) => {
                let context = self.context()?;
                let connect = AeronArchiveAsyncConnect::new_with_aeron(&context, &self.aeron)
                    .map_err(|e| format!("connecting to the archive: {e}"))?;
                Stage::Connecting(connect, context)
            }
            Stage::Connecting(connect, context) => {
                match connect.poll().map_err(|e| format!("connecting to the archive: {e}"))? {
                    None => Stage::Connecting(connect, context),
                    Some(archive) => self.list(&archive)?,
                }
            }
            Stage::Listing(mut request, newest) => {
                match request.poll().map_err(|e| format!("listing recordings: {e}"))? {
                    None => Stage::Listing(request, newest),
                    Some(_) => match newest.get() {
                        Some((recording_id, session_id)) => self.subscribe(recording_id, session_id)?,
                        None => return Err("no recording to follow yet".to_owned()),
                    },
                }
            }
            Stage::Building(mut building, recording_id) => {
                match building
                    .poll()
                    .map_err(|e| format!("subscribing to recording {recording_id}: {e}"))?
                {
                    None => Stage::Building(building, recording_id),
                    Some(subscription) => {
                        log::info!("following recording {recording_id} of stream {}", self.stream_id);
                        self.active_at = Instant::now();
                        Stage::Following {
                            subscription,
                            recording_id,
                        }
                    }
                }
            }
            following @ Stage::Following { .. } => following,
        })
    }

    fn context(&self) -> Result<AeronArchiveContext, String> {
        (self.archive_context)().map_err(|e| format!("making an archive context: {e}"))
    }

    fn list(&self, archive: &AeronArchive) -> Result<Stage, String> {
        let newest = Rc::new(Cell::new(None));
        let (found, filter) = (Rc::clone(&newest), Rc::clone(&self.filter));
        let consumer: ListConsumer = Box::new(move |recording| {
            if filter(&recording) {
                found.set(
                    found
                        .get()
                        .max(Some((recording.recording_id(), recording.session_id()))),
                );
            }
        });
        let request = archive
            .async_list_recordings_for_uri(0, i32::MAX, &self.channel_fragment, self.stream_id, consumer)
            .map_err(|e| format!("listing recordings: {e}"))?;
        Ok(Stage::Listing(request, newest))
    }

    fn subscribe(&mut self, recording_id: i64, session_id: i32) -> Result<Stage, String> {
        let start = start_position(self.resume, recording_id, self.from_start);
        self.resume = Some((recording_id, start));
        let context = self.context()?;
        let building = PersistentSubscriptionBuilder::new()
            .and_then(|b| b.aeron(&self.aeron))
            .and_then(|b| b.archive_context(&context))
            .and_then(|b| b.recording_id(recording_id))
            .and_then(|b| b.live_channel(&ChannelUri::add_session_id(&self.live_channel, session_id)))
            .and_then(|b| b.live_stream_id(self.stream_id))
            .and_then(|b| b.replay_channel(&self.replay_channel))
            .and_then(|b| b.replay_stream_id(self.replay_stream_id))
            .and_then(|b| b.start_position(start))
            .and_then(PersistentSubscriptionBuilder::build_async)
            .map_err(|e| format!("subscribing to recording {recording_id}: {e}"))?;
        Ok(Stage::Building(building, recording_id))
    }
}

/// Where to start `recording_id`: where the last one followed was left if it is the same,
/// else from its start, or from live for a first recording not followed `from_start`.
fn start_position(resume: Option<(i64, i64)>, recording_id: i64, from_start: bool) -> i64 {
    match resume {
        Some((followed, position)) if followed == recording_id => position,
        Some(_) => PERSISTENT_SUBSCRIPTION_FROM_START,
        None if from_start => PERSISTENT_SUBSCRIPTION_FROM_START,
        None => PERSISTENT_SUBSCRIPTION_FROM_LIVE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resumes_the_same_recording_and_replays_a_new_one_from_its_start() {
        assert_eq!(start_position(Some((7, 4_096)), 7, false), 4_096);
        assert_eq!(
            start_position(Some((7, 4_096)), 8, false),
            PERSISTENT_SUBSCRIPTION_FROM_START
        );
        assert_eq!(start_position(None, 7, true), PERSISTENT_SUBSCRIPTION_FROM_START);
        assert_eq!(start_position(None, 7, false), PERSISTENT_SUBSCRIPTION_FROM_LIVE);
    }
}
