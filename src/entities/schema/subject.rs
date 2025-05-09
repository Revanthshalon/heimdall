use std::sync::Arc;

#[derive(Debug)]
pub struct SubjectSetRewrite {
    pub operation: Operator,
    pub children: Arc<Vec<Child>>,
}

impl Clone for SubjectSetRewrite {
    fn clone(&self) -> Self {
        SubjectSetRewrite {
            operation: self.operation,
            children: Arc::clone(&self.children),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operator {
    And,
    Or,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::And => write!(f, "and"),
            Self::Or => write!(f, "or"),
        }
    }
}

#[derive(Debug)]
pub enum Child {
    Rewrite {
        rewrite: Arc<SubjectSetRewrite>,
    },
    ComputerSubjectSet {
        relation: Arc<str>,
    },
    TupleToSubjectSet {
        relation: Arc<str>,
        computed_subject_set_relation: Arc<str>,
    },
    InvertResult {
        child: Arc<Child>,
    },
}

impl Clone for Child {
    fn clone(&self) -> Self {
        match self {
            Self::Rewrite { rewrite } => Self::Rewrite {
                rewrite: Arc::clone(rewrite),
            },
            Self::ComputerSubjectSet { relation } => Self::ComputerSubjectSet {
                relation: Arc::clone(relation),
            },
            Self::TupleToSubjectSet {
                relation,
                computed_subject_set_relation,
            } => Self::TupleToSubjectSet {
                relation: Arc::clone(relation),
                computed_subject_set_relation: Arc::clone(computed_subject_set_relation),
            },
            Self::InvertResult { child } => Self::InvertResult {
                child: Arc::clone(child),
            },
        }
    }
}

impl Child {
    pub fn as_rewrite(&self) -> Arc<SubjectSetRewrite> {
        match self {
            Self::Rewrite { rewrite } => rewrite.clone(),
            _ => Arc::new(SubjectSetRewrite {
                operation: Operator::And,
                children: Arc::new(vec![self.clone()]),
            }),
        }
    }
}
