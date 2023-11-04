mod test;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Limits<T: std::cmp::Eq + std::hash::Hash + std::marker::Copy> {
    pub limits: HashMap<T, Vec<Limit>>,
    pub limit_cache: HashMap<u64, u64>,
}

#[derive(Debug)]
pub struct Limit {
    pub cost: u64,
    pub time: u64,
}

impl<T: std::cmp::Eq + std::hash::Hash + std::marker::Copy> Limits<T> {
    pub fn new() -> Limits<T> {
        Limits {
            limits: HashMap::new(),
            limit_cache: HashMap::new(),
        }
    }

    pub fn add_category(&mut self, category: T) {
        self.limits.insert(category, Vec::new());
        self.update_limit(category, u64::max_value(), u64::max_value());
    }

    pub fn is_initialized(&self) -> bool {
        // limits must contain at least category and each category must have at least one limit
        self.limits.len() > 0 && self.limits.values().all(|v| v.len() > 0)
    }

    pub fn update_limit(&mut self, category: T, cost: u64, time: u64) -> bool {
        let limit = Limit { cost, time };
        let limits = self.limits.get_mut(&category).unwrap();
        // check if any limit dominates the new limit
        for l in limits.iter() {
            if l.cost <= limit.cost && l.time <= limit.time {
                return false;
            }
        }
        // remove all limits dominated by the new limit
        limits.retain(|l| l.cost > limit.cost || l.time > limit.time);

        limits.push(limit);

        self.limit_cache.clear();

        return true;
    }

    /// is_limit_exceeded returns true if each category has a limit that dominates the given cost and time
    pub fn is_limit_exceeded(&mut self, cost: u64, time: u64) -> bool {
        if let Some(&limit) = self.limit_cache.get(&cost) {
            return limit <= time;
        }
        let limit = self.determine_limit(cost);
        self.limit_cache.insert(cost, limit);
        return limit <= time;
    }

    fn determine_limit(&mut self, cost: u64) -> u64 {
        let mut min_limits = Vec::new();
        for limits in self.limits.values() {
            let mut min_limit = u64::max_value();
            for limit in limits.iter() {
                if limit.cost <= cost {
                    min_limit = std::cmp::min(min_limit, limit.time);
                }
            }
            min_limits.push(min_limit);
        }
        return min_limits.iter().max().unwrap().clone();
    }
}
