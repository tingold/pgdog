use crate::config::{ShardListMap, ShardRangeMap};
use super::Centroids;

#[derive(Debug)]
pub enum Operator<'a> {
    Shards(usize),
    Centroids {
        shards: usize,
        probes: usize,
        centroids: Centroids<'a>,
    },
    Lists(ShardListMap),
    Ranges(ShardRangeMap),

}
