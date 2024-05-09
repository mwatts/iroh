use ed25519_dalek::SignatureError;

use crate::{
    proto::{meadowcap::InvalidCapability, wgps::ResourceHandle, willow::Unauthorised},
    util::channel::{ReadError, WriteError},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("local store failed")]
    Store(#[from] anyhow::Error),
    #[error("failed to receive data: {0}")]
    Receive(#[from] ReadError),
    #[error("failed to send data: {0}")]
    Write(#[from] WriteError),
    #[error("wrong secret key for capability")]
    WrongSecretKeyForCapability,
    #[error("missing resource {0:?}")]
    MissingResource(ResourceHandle),
    #[error("received capability is invalid")]
    InvalidCapability,
    #[error("received capability has an invalid signature")]
    InvalidSignature,
    #[error("missing resource")]
    RangeOutsideCapability,
    #[error("received a message that is not valid in the current session state")]
    InvalidMessageInCurrentState,
    #[error("our and their area of interests refer to different namespaces")]
    AreaOfInterestNamespaceMismatch,
    #[error("our and their area of interests do not overlap")]
    AreaOfInterestDoesNotOverlap,
    #[error("received an entry which is not authorised")]
    UnauthorisedEntryReceived,
    #[error("received an unsupported message type")]
    UnsupportedMessage,
    #[error("the received nonce does not match the received committment")]
    BrokenCommittement,
    #[error("received an actor message for unknown session")]
    SessionNotFound,
    #[error("invalid parameters: {0}")]
    InvalidParameters(&'static str),
}

impl From<Unauthorised> for Error {
    fn from(_value: Unauthorised) -> Self {
        Self::UnauthorisedEntryReceived
    }
}
impl From<InvalidCapability> for Error {
    fn from(_value: InvalidCapability) -> Self {
        Self::InvalidCapability
    }
}

impl From<SignatureError> for Error {
    fn from(_value: SignatureError) -> Self {
        Self::InvalidSignature
    }
}