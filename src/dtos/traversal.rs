#![allow(unused)]

use serde::{Deserialize, Serialize};

use crate::errors::HeimdallResult;

use super::relation_tuple::{RelationTuple, Subject};

pub struct TraversalResult {
    pub from: RelationTuple,
    pub to: RelationTuple,
    pub via: TraversalType,
    pub found: bool,
}

pub trait TraverserTrait {
    fn traverse_subject_set_expansion(
        &self,
        tuple: RelationTuple,
    ) -> HeimdallResult<Vec<TraversalResult>>;

    fn traverse_subject_set_rewrite(
        &self,
        tuple: RelationTuple,
        computed_subject_sets: Vec<String>,
    ) -> HeimdallResult<Vec<TraversalResult>>;
}

#[derive(Debug, PartialEq)]
pub enum TraversalType {
    Unknown,
    SubjectSetExpand,
    ComputerUserset,
    TupleToUserset,
}

impl std::fmt::Display for TraversalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraversalType::Unknown => write!(f, "unknown"),
            TraversalType::SubjectSetExpand => write!(f, "subject set expand"),
            TraversalType::ComputerUserset => write!(f, "computed userset"),
            TraversalType::TupleToUserset => write!(f, "tuple to userset"),
        }
    }
}

pub struct Tree {
    pub subject: Subject,
    pub children: Vec<Tree>,
}
