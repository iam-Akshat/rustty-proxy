use std::collections::HashMap;

use rand::prelude::*;
use rand_distr::{weighted_alias, WeightedAliasIndex, WeightedError};

#[derive(Debug, Clone)]
pub struct LoadBalancer {
    pub target_weight: HashMap<String, u16>,
    weighted_alias: WeightedAliasIndex<u16>,
    pub targets: Vec<String>,
    pub is_active: bool,
}

impl LoadBalancer {
    pub fn new(targets: &Vec<String>, weights: &Vec<u16>) -> LoadBalancer {
        let weighted_alias = weighted_alias::WeightedAliasIndex::new(weights.to_vec()).unwrap();
        let mut target_weight = HashMap::new();
        for (i, target) in targets.iter().enumerate() {
            target_weight.insert(target.to_string(), weights[i]);
        }
        LoadBalancer {
            target_weight,
            weighted_alias,
            targets: targets.to_vec(),
            is_active: true,
        }
    }
    pub fn get_target(&self) -> String {
        //
        let index = self.weighted_alias.sample(&mut rand::thread_rng());
        self.target_weight.keys().nth(index).unwrap().to_string()
    }
    pub fn update_weight(&mut self, target: String, weight: u16) {
        self.target_weight.insert(target, weight);
        let mut weights = Vec::new();
        for (_, weight) in self.target_weight.iter() {
            weights.push(*weight);
        }
        self.weighted_alias = match weighted_alias::WeightedAliasIndex::new(weights.clone()) {
            Ok(w) => {
                self.is_active = true;
                w
            }
            Err(e) => match e {
                WeightedError::AllWeightsZero => {
                    self.is_active = false;
                    weighted_alias::WeightedAliasIndex::new(weights).unwrap()
                }
                _ => panic!("Error: {:?}", e),
            },
        };
    }

    pub fn print_targets_state(&self) {
        for (target, weight) in self.target_weight.iter() {
            println!("{}: ->>>>>> {}", target, weight);
        }
    }
}
