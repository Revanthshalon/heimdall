#![allow(unused)]

use super::relation_tuple::{RelationTuples, Subject};

/// Represents different types of traversal operations in the authorization system.
///
/// This enum is used to categorize the method by which relationships are being traversed
/// when evaluating permissions.
#[derive(Debug, PartialEq)]
pub enum TraversalType {
    /// Represents an unspecified or invalid traversal type.
    Unknown,
    /// Expands a subject set to find all members.
    SubjectSetExpand,
    /// Computes a user set based on rules or relationships.
    ComputedUserSet,
    /// Traverses from a tuple to a userset.
    TupleToUserset,
}

impl std::fmt::Display for TraversalType {
    /// Formats the traversal type as a human-readable string.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write the string to.
    ///
    /// # Returns
    ///
    /// A Result indicating whether the formatting operation succeeded.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraversalType::Unknown => write!(f, "unknown"),
            TraversalType::SubjectSetExpand => write!(f, "subject set expand"),
            TraversalType::ComputedUserSet => write!(f, "computed userset"),
            TraversalType::TupleToUserset => write!(f, "tuple to userset"),
        }
    }
}

/// Represents the result of a traversal operation in the permission graph.
///
/// Contains information about the source and destination relation tuples,
/// the traversal method used, and whether the traversal was successful.
#[derive(Debug)]
pub struct TraversalResult {
    /// The source relation tuple where traversal began.
    pub from: RelationTuples,
    /// The destination relation tuple where traversal ended.
    pub to: RelationTuples,
    /// The method used for traversal.
    pub via: TraversalType,
    /// Indicates whether the traversal was successful in finding a path.
    pub found: bool,
}

/// A hierarchical structure representing subject relationships.
///
/// Used to model nested subject relationships in a tree format, where
/// each node can have multiple child nodes.
pub struct Tree {
    /// The subject at this node in the tree.
    pub subject: Subject,
    /// Child nodes representing related subjects.
    pub children: Vec<Tree>,
}
