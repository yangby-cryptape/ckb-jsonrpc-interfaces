// Copyright (C) 2019 Boyu Yang
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::convert::TryInto;
use std::sync::Arc;

use jsonrpc_sdk_client::sync::Client;
use jsonrpc_sdk_prelude::{Error, Result};

use ckb_jsonrpc_interfaces::{core, types, Ckb, OccupiedCapacity, H256};

pub struct CkbClient {
    cli: Arc<Client>,
    url: Arc<String>,
}

impl CkbClient {
    pub fn new(url: &str) -> Self {
        Self {
            cli: Arc::new(Client::new()),
            url: Arc::new(url.to_owned()),
        }
    }

    /*
     * Basic
     */

    pub fn tip_block_number(&self) -> Result<core::BlockNumber> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_tip_block_number(), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: String| {
                r.parse()
                    .map_err(|_| Error::custom("parse block number failed"))
            })
    }

    pub fn tip_header(&self) -> Result<types::Header> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_tip_header(), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn block_hash(&self, height: Option<core::BlockNumber>) -> Result<H256> {
        let cli = Arc::clone(&self.cli);
        let url = Arc::clone(&self.url);
        if let Some(h) = height {
            Ok(h)
        } else {
            self.tip_block_number()
        }
        .and_then(move |h| {
            cli.post(&*url)
                .send(Ckb::get_block_hash(h.to_string()), Default::default())
                .map(std::convert::Into::into)
                .and_then(|r: Option<H256>| {
                    r.ok_or_else(|| Error::custom("fetch block hash failed"))
                })
        })
    }

    pub fn block_by_number(&self, height: Option<core::BlockNumber>) -> Result<types::Block> {
        let cli = Arc::clone(&self.cli);
        let url = Arc::clone(&self.url);
        self.block_hash(height).and_then(move |r| {
            cli.post(&*url)
                .send(Ckb::get_block(r), Default::default())
                .map(std::convert::Into::into)
                .and_then(|r: Option<types::Block>| {
                    r.ok_or_else(|| Error::custom("fetch block failed"))
                })
        })
    }

    pub fn block_by_hash(&self, hash: H256) -> Result<types::Block> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_block(hash), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: Option<types::Block>| {
                r.ok_or_else(|| Error::custom("fetch block failed"))
            })
    }

    pub fn genesis_block(&self) -> Result<types::Block> {
        self.block_by_number(Some(0))
    }

    pub fn last_block(&self) -> Result<types::Block> {
        self.block_by_number(None)
    }

    pub fn cells_by_lock_hash(
        &self,
        lock: &core::script::Script,
        from: Option<core::BlockNumber>,
        to: Option<core::BlockNumber>,
    ) -> Result<Vec<types::CellOutputWithOutPoint>> {
        let lock_hash = lock.hash();
        let from = from.unwrap_or(0);
        let cli = Arc::clone(&self.cli);
        let url = Arc::clone(&self.url);
        if let Some(h) = to {
            Ok(h)
        } else {
            self.tip_block_number()
        }
        .and_then(move |to| {
            cli.post(&*url)
                .send(
                    Ckb::get_cells_by_lock_hash(lock_hash, from.to_string(), to.to_string()),
                    Default::default(),
                )
                .map(std::convert::Into::into)
        })
    }

    pub fn live_cell(&self, out_point: types::OutPoint) -> Result<types::CellWithStatus> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_live_cell(out_point), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn total_capacity(&self, lock: &core::script::Script) -> Result<u64> {
        self.cells_by_lock_hash(lock, None, None).and_then(|u| {
            u.into_iter()
                .map(|c| c.capacity.parse::<u64>())
                .collect::<::std::result::Result<Vec<_>, std::num::ParseIntError>>()
                .map_err(|_| Error::custom("parse capacity failed"))
                .and_then(|caps| {
                    caps.into_iter()
                        .try_fold(0u64, u64::checked_add)
                        .ok_or_else(|| Error::custom("sum capacity overflow"))
                })
        })
    }

    pub fn send(&self, tx: types::Transaction) -> Result<H256> {
        self.cli
            .post(&*self.url)
            .send(Ckb::send_transaction(tx), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn pool_transaction(&self, hash: H256) -> Result<types::Transaction> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_pool_transaction(hash), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: Option<types::Transaction>| {
                r.ok_or_else(|| Error::custom("fetch pool transaction failed"))
            })
    }

    pub fn transaction(&self, hash: H256) -> Result<types::Transaction> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_transaction(hash), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: Option<types::Transaction>| {
                r.ok_or_else(|| Error::custom("fetch transaction failed"))
            })
    }

    pub fn trace(&self, tx: types::Transaction) -> Result<H256> {
        self.cli
            .post(&*self.url)
            .send(Ckb::trace_transaction(tx), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn transaction_trace(&self, hash: H256) -> Result<Vec<types::TxTrace>> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_transaction_trace(hash), Default::default())
            .map(std::convert::Into::into)
            .and_then(|r: Option<Vec<types::TxTrace>>| {
                r.ok_or_else(|| Error::custom("fetch transaction trace failed"))
            })
    }

    pub fn local_node_info(&self) -> Result<types::Node> {
        self.cli
            .post(&*self.url)
            .send(Ckb::local_node_info(), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn get_peers(&self) -> Result<Vec<types::Node>> {
        self.cli
            .post(&*self.url)
            .send(Ckb::get_peers(), Default::default())
            .map(std::convert::Into::into)
    }

    pub fn add_node(&self, peer_id: String, address: String) -> Result<()> {
        self.cli
            .post(&*self.url)
            .send(Ckb::add_node(peer_id, address), Default::default())
            .map(std::convert::Into::into)
    }

    /*
     * Combine
     */

    pub fn gather(
        &self,
        lock_in: &core::script::Script,
        lock_out: &core::script::Script,
        from: Option<core::BlockNumber>,
        to: Option<core::BlockNumber>,
    ) -> Result<types::Transaction> {
        let lock_out = lock_out.clone();
        self.cells_by_lock_hash(lock_in, from, to).and_then(
            move |cells: Vec<types::CellOutputWithOutPoint>| {
                let capacity = cells
                    .iter()
                    .map(|c| c.capacity.parse::<u64>())
                    .collect::<::std::result::Result<Vec<_>, std::num::ParseIntError>>()
                    .map_err(|_| Error::custom("parse capacity failed"))
                    .and_then(|caps| {
                        caps.into_iter()
                            .try_fold(0u64, u64::checked_add)
                            .ok_or_else(|| Error::custom("sum capacity overflow"))
                    })?;

                let inputs = cells
                    .into_iter()
                    .map(|c| {
                        core::transaction::CellInput {
                            previous_output: c.out_point.try_into().unwrap(),
                            args: vec![],
                            since: 0,
                        }
                        .into()
                    })
                    .collect();
                let output = core::transaction::CellOutput::new(
                    core::Capacity::shannons(capacity),
                    Vec::new(),
                    lock_out,
                    None,
                );
                Ok(types::Transaction {
                    version: 0,
                    deps: vec![],
                    inputs,
                    outputs: vec![output.into()],
                    witnesses: vec![],
                    hash: Default::default(),
                })
            },
        )
    }

    pub fn disperse(
        &self,
        lock_in: &core::script::Script,
        lock_out: &core::script::Script,
        from: Option<core::BlockNumber>,
        to: Option<core::BlockNumber>,
        max_count: usize,
    ) -> Result<types::Transaction> {
        let lock_out = lock_out.clone();
        self.cells_by_lock_hash(lock_in, from, to)
            .and_then(|cells| {
                if cells.is_empty() {
                    Err(Error::custom("input is empty"))
                } else {
                    Ok(cells)
                }
            })
            .and_then(move |cells: Vec<types::CellOutputWithOutPoint>| {
                let mut capacity = cells
                    .iter()
                    .map(|c| c.capacity.parse::<u64>())
                    .collect::<::std::result::Result<Vec<_>, std::num::ParseIntError>>()
                    .map_err(|_| Error::custom("parse capacity failed"))
                    .and_then(|caps| {
                        caps.into_iter()
                            .try_fold(0u64, u64::checked_add)
                            .ok_or_else(|| Error::custom("sum capacity overflow"))
                    })?;

                let inputs = cells
                    .into_iter()
                    .map(|c| {
                        core::transaction::CellInput {
                            previous_output: c.out_point.try_into().unwrap(),
                            args: vec![],
                            since: 0,
                        }
                        .into()
                    })
                    .collect();
                let mut outputs = Vec::new();
                while capacity > 0 && outputs.len() < max_count {
                    let mut output = core::transaction::CellOutput::new(
                        core::Capacity::shannons(0),
                        Vec::new(),
                        lock_out.clone(),
                        None,
                    );
                    output.capacity = output
                        .occupied_capacity()
                        .map_err(|_| Error::custom("capacity overflow"))?;
                    if capacity < output.capacity.as_u64() {
                        break;
                    }
                    capacity -= output.capacity.as_u64();
                    outputs.push(output);
                }
                if capacity > 0 {
                    outputs[0].capacity = outputs[0]
                        .capacity
                        .safe_add(core::Capacity::shannons(capacity))
                        .map_err(|_| Error::custom("capacity overflow"))?;
                }
                Ok(types::Transaction {
                    version: 0,
                    deps: vec![],
                    inputs,
                    outputs: outputs.into_iter().map(Into::into).collect(),
                    witnesses: vec![],
                    hash: Default::default(),
                })
            })
    }
}
