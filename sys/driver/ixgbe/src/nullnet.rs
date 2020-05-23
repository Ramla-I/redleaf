use alloc::collections::VecDeque;
use alloc::vec::Vec;
use rref::{RRef, RRefDeque};
use usr::rpc::RpcResult;
use usr::error::Result;
use crate::NetworkStats;

pub struct NullNet {}

impl NullNet {
    pub fn new() -> Self {
        Self{}
    }
}

impl usr::net::Net for NullNet {
    fn submit_and_poll(&self, mut packets: &mut VecDeque<Vec<u8>
        >, mut collect: &mut VecDeque<Vec<u8>>, tx: bool) -> RpcResult<Result<usize>> {
        unimplemented!()
    }

    fn submit_and_poll_rref(
        &self,
        mut packets: RRefDeque<[u8; 1512], 32>,
        mut collect: RRefDeque<[u8; 1512], 32>,
        tx: bool,
        pkt_len: usize) -> RpcResult<Result<(
            usize,
            RRefDeque<[u8; 1512], 32>,
            RRefDeque<[u8; 1512], 32>
        )>>
    {
        Ok(Ok((packets.len(), collect, packets)))
    }

    fn poll(&self, mut collect: &mut VecDeque<Vec<u8>>, tx: bool) -> RpcResult<Result<usize>> {
        unimplemented!()
    }

    fn poll_rref(&self, collect: RRefDeque<[u8; 1512], 512>, tx: bool) -> RpcResult<Result<(usize, RRefDeque<[u8; 1512], 512>)>> {
        Ok(Ok((0, collect)))
    }

    fn get_stats(&self) -> RpcResult<Result<NetworkStats>> {
        Ok(Ok(NetworkStats::new()))
    }
}