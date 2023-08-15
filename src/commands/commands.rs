use std::sync::Arc;

use num_derive::FromPrimitive;
use parking_lot::RwLock;

use crate::state::{datastate::DataState, serverstate::ServerState};

use super::implcommands::{
    delete::DeleteCmd,
    get::GetCmd,
    hll::{hlladd::HLLAddCmd, hllcount::HLLCountCmd, hllreset::HLLResetCmd},
    incrf::IncrF,
    incri::IncrI,
    info::InfoCmd,
    list::{
        lextract::LExtractCmd, llen::LLenCmd, lpop::LPopCmd, lpush::LPushCmd, lrange::LRangeCmd,
    },
    setf::SetF,
    seti::SetI,
    sets::SetSCmd,
    test::TestCmd,
};

#[derive(Debug)]
pub struct Command<'a> {
    pub command_type: CommandType,
    pub arguments: Vec<&'a [u8]>,
}
impl Command<'_> {
    pub fn execute<'a>(
        self: &Self,
        data_state: &Arc<DataState>,
        server_state_rwl: &Arc<RwLock<ServerState>>,
    ) -> Result<Option<Vec<u8>>, String> {
        match self.command_type {
            CommandType::Info => InfoCmd::execute(server_state_rwl),
            CommandType::Test => TestCmd::execute(),
            CommandType::SetString => SetSCmd::execute(data_state, self),
            CommandType::SetInt => SetI::execute(data_state, self),
            CommandType::SetFloat => SetF::execute(data_state, self),
            CommandType::IncrementInt => IncrI::execute(data_state, self),
            CommandType::IncrementFloat => IncrF::execute(data_state, self),
            CommandType::Get => GetCmd::execute(data_state, self),
            CommandType::Delete => DeleteCmd::execute(data_state, self),
            CommandType::ListPush => LPushCmd::execute(data_state, self),
            CommandType::ListExtract => LExtractCmd::execute(data_state, self),
            CommandType::ListPop => LPopCmd::execute(data_state, self),
            CommandType::ListRange => LRangeCmd::execute(data_state, self),
            CommandType::ListLength => LLenCmd::execute(data_state, self),
            CommandType::HLLAdd => HLLAddCmd::execute(data_state, self),
            CommandType::HLLCount => HLLCountCmd::execute(data_state, self),
            CommandType::HLLReset => HLLResetCmd::execute(data_state, self),
            _ => Err("Unknown command".to_owned()),
        }
    }
}

#[derive(FromPrimitive, Debug)]
#[repr(u16)]
pub enum CommandType {
    Info,
    Test,
    SetString,
    SetInt,
    SetFloat,
    Get,
    Delete,
    IncrementInt,
    IncrementFloat,
    ListPush,
    ListPop,
    ListRange,
    ListExtract,
    ListLength,
    HLLAdd,
    HLLCount,
    HLLReset,
    Unknown,
}
impl From<[u8; 2]> for CommandType {
    fn from(value: [u8; 2]) -> Self {
        let num = u16::from_le_bytes([value[0], value[1]]);
        let element = num::FromPrimitive::from_u16(num);
        match element {
            Some(cmd) => cmd,
            _ => CommandType::Unknown,
        }
    }
}
