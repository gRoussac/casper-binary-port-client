use casper_binary_port::{
    BinaryRequest, BinaryResponseAndRequest, GetRequest, GlobalStateRequest, PayloadType,
};
use casper_types::{Digest, GlobalStateIdentifier, Key};
use clap::Subcommand;

use crate::{communication::send_request, error::Error, utils::print_hex_payload};

#[derive(Debug, Subcommand)]
pub(crate) enum State {
    Item {
        #[clap(long, conflicts_with = "block_hash", conflicts_with = "block_height")]
        state_root_hash: Option<String>,
        #[clap(
            long,
            conflicts_with = "block_height",
            conflicts_with = "state_root_hash"
        )]
        block_hash: Option<String>,
        #[clap(
            long,
            conflicts_with = "block_hash",
            conflicts_with = "state_root_hash"
        )]
        block_height: Option<u64>,
        #[clap(long, short)]
        base_key: String,
        #[clap(long, short)]
        path: Option<String>,
    },
}

impl TryFrom<State> for GlobalStateRequest {
    type Error = Error;

    fn try_from(value: State) -> Result<Self, Self::Error> {
        match value {
            State::Item {
                state_root_hash,
                block_hash,
                block_height,
                base_key,
                path,
            } => {
                let state_identifier = match (state_root_hash, block_hash, block_height) {
                    (Some(state_root_hash), None, None) => {
                        let digest = Digest::from_hex(state_root_hash)?;
                        Some(GlobalStateIdentifier::StateRootHash(digest))
                    },
                    (None, Some(block_hash), None) => {
                        let digest = Digest::from_hex(block_hash)?;
                        Some(GlobalStateIdentifier::BlockHash(digest.into()))
                    },
                    (None, None, Some(block_height)) => Some(GlobalStateIdentifier::BlockHeight(block_height)),
                    (None, None, None) => None,
                    _ => unreachable!("global state must either be empty or be identified by exactly one of: state_root_hash, block_hash, block_height"),
                };
                let base_key =
                    Key::from_formatted_str(&base_key).map_err(|err| Error::KeyFromStr(err))?;
                Ok(GlobalStateRequest::Item {
                    state_identifier,
                    base_key,
                    path: vec![],
                })
            }
        }
    }
}

pub(super) async fn handle_state_request(req: State) -> Result<(), Error> {
    let request: GlobalStateRequest = req.try_into()?;
    let response = send_request(BinaryRequest::Get(GetRequest::State(Box::new(request)))).await?;
    handle_state_response(&response);

    Ok(())
}

fn handle_state_response(response: &BinaryResponseAndRequest) {
    assert_eq!(
        Some(PayloadType::GlobalStateQueryResult as u8),
        response.response().returned_data_type_tag(),
        "should get GlobalStateQueryResult"
    );
    print_hex_payload(response.response().payload());
}