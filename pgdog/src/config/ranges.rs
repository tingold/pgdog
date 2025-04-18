use std::collections::{BTreeMap, HashMap};
use std::ops::Bound;
use std::str::FromStr;
use std::usize;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ShardRangeHelper {
    // Use string keys in the initial parse
    pub range_map: HashMap<String, Vec<IntRange>>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(from = "ShardRangeHelper",)]
pub struct ShardRanges {
    // Original range map using i8 keys
    pub range_map: HashMap<usize, Vec<IntRange>>,
    // Optimized lookup structure
    lookup: BTreeMap<i64, (i64, usize)>,
}
impl ShardRanges {
    pub fn from_toml(toml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let helper: ShardRangeHelper = toml::from_str(toml_str)?;

        // Convert string keys to i8
        let mut range_map = HashMap::new();
        for (key_str, value) in &helper.range_map {
            let key = usize::from_str(key_str)?;
            range_map.insert(key, value.clone());
        }

        // Create the optimized lookup BTreeMap
        let mut lookup = BTreeMap::new();
        for (&shard_key, ranges) in &range_map {
            for range in ranges {
                let min_val = match range.min {
                    Some(min) => min,
                    None => i64::MIN,
                };

                let max_val = match range.max {
                    Some(max) => max,
                    None => i64::MAX,
                };

                lookup.insert(min_val, (max_val, shard_key));
            }
        }

        Ok(ShardRanges {
            range_map,
            lookup
        })
    }

    /// Find the shard key for a given integer value
    pub fn find_shard_for_value(&self, value: i64) -> Option<usize> {
        // Use BTreeMap's range to find potential ranges that include our value
        // We look for ranges where start ≤ value
        let candidates = self.lookup.range((Bound::Unbounded, Bound::Included(value)));

        // Check each candidate range to see if our value falls within it
        for (&range_start, &(range_end, shard_key)) in candidates {
            if value >= range_start && value <= range_end {
                return Some(shard_key);
            }
        }

        None
    }

    /// Find all shard keys where the value falls within their range
    /// This is useful if ranges can overlap
    pub fn find_all_shards_for_value(&self, value: i64) -> Vec<usize> {
        let candidates = self.lookup.range((Bound::Unbounded, Bound::Included(value)));
        let mut result = Vec::new();

        for (&range_start, &(range_end, shard_key)) in candidates {
            if value >= range_start && value <= range_end {
                result.push(shard_key);
            }
        }

        result
    }

    /// Get a reference to the original range map
    pub fn get_range_map(&self) -> &HashMap<usize, Vec<IntRange>> {
        &self.range_map
    }

    /// Create a new ShardRange from an existing HashMap
    pub fn new(range_map: HashMap<usize, Vec<IntRange>>) -> Self {
        let mut lookup = BTreeMap::new();

        for (&shard_key, ranges) in &range_map {
            for range in ranges {
                let min_val = match range.min {
                    Some(min) => min,
                    None => i64::MIN,
                };

                let max_val = match range.max {
                    Some(max) => max,
                    None => i64::MAX,
                };

                lookup.insert(min_val, (max_val, shard_key));
            }
        }

        ShardRanges { range_map, lookup }
    }
}

impl From<ShardRangeHelper> for ShardRanges {
    fn from(helper: ShardRangeHelper) -> Self {
        let mut range_map = HashMap::new();

        // Convert string keys to i8
        for (key_str, value) in helper.range_map {
            if let Ok(key) = usize::from_str(&key_str) {
                range_map.insert(key, value);
            }
            // You might want to handle errors differently here
        }

        // Create the optimized lookup BTreeMap
        let mut lookup = BTreeMap::new();
        for (&shard_key, ranges) in &range_map {
            for range in ranges {
                let min_val = match range.min {
                    Some(min) => min,
                    None => i64::MIN,
                };

                let max_val = match range.max {
                    Some(max) => max,
                    None => i64::MAX,
                };

                lookup.insert(min_val, (max_val, shard_key));
            }
        }

        ShardRanges { range_map, lookup }
    }
}

// Now you can also implement Deserialize for ShardRange
impl<'de> Deserialize<'de> for ShardRanges {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize into helper first
        let helper = ShardRangeHelper::deserialize(deserializer)?;
        // Then convert using the From trait we implemented
        Ok(ShardRanges::from(helper))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IntRanges {
    pub ranges: Vec<IntRange>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct IntRange {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub no_max: Option<bool>,
    pub no_min: Option<bool>,
}
impl IntRange {
    /// Get the effective minimum value of the range
    pub fn effective_min(&self) -> i64 {
        match self.min {
            Some(min) => min,
            None => if self.no_min.unwrap_or(false) { i64::MIN } else { i64::MIN }
        }
    }

    /// Get the effective maximum value of the range
    pub fn effective_max(&self) -> i64 {
        match self.max {
            Some(max) => max,
            None => if self.no_max.unwrap_or(false) { i64::MAX } else { i64::MAX }
        }
    }

    /// Check if a value is within this range
    pub fn contains(&self, value: i64) -> bool {
        let min_match = match self.min {
            Some(min) => value >= min,
            None => self.no_min.unwrap_or(false) || true
        };

        let max_match = match self.max {
            Some(max) => value <= max,
            None => self.no_max.unwrap_or(false) || true
        };

        min_match && max_match
    }
}